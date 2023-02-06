mod protocol;

use serde::Deserialize;

use std::io::Write;
use std::sync::Arc;
use std::{error::Error, path::Path};

use chrono::Utc;
use prost::Message;
use url::Url;

use crate::protocol::{
    DailyStoic, DailyStoicRequest, DAILY_STOIC_CONTENT_TOPIC, DAILY_STOIC_REQUEST_CONTENT_TOPIC,
};
use waku_bindings::{
    waku_new, waku_set_event_callback, ProtocolId, Running, WakuMessage, WakuNodeHandle,
};

#[derive(Deserialize, Debug)]
struct Quote {
    author: String,
    quote: String,
}

struct App {
    pub node_handle: WakuNodeHandle<Running>,
    pub quotes: Vec<Quote>,
}

/// The enrtree address of the production waku2 fleet
pub static ENRTREE: &str =
    "enrtree://AOGECG2SPND25EEFMAJ5WF3KSGJNSGV356DSTL2YVLLZWIV6SAYBM@prod.waku.nodes.status.im";

/// Setup a waku node and connect to the waku fleet
fn setup_node_handle() -> Result<WakuNodeHandle<Running>, Box<dyn Error>> {
    let node_handle = waku_new(None)?;
    let node_handle = node_handle.start()?;

    // Get the addresses of the waku fleet via the enrtree
    let addresses = node_handle.dns_discovery(&Url::parse(ENRTREE)?, None, None)?;

    // Iterate over the addresses and connect to the waku fleet
    for address in addresses {
        let peer_id = node_handle.add_peer(&address, ProtocolId::Relay)?;
        node_handle
            .connect_peer_with_id(peer_id, None)
            .unwrap_or_else(|e| {
                let mut out = std::io::stderr();
                write!(out, "Error, couldn't connect to peer {e:?}").unwrap();
            });
    }

    node_handle.relay_subscribe(None)?;
    Ok(node_handle)
}

#[tokio::main]
async fn main() {
    // The first and only argument is a path to a file containing stoic quotes
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage: dailystoic <path to file containing stoic quotes>");
        return;
    }

    let path = &args[1];
    if !std::path::Path::new(path).exists() {
        println!("File {path} does not exist");
        return;
    }

    // Create an app instance
    let app = Arc::new(App {
        node_handle: setup_node_handle().unwrap(),
        quotes: read_quotes_from_file(path).unwrap(),
    });

    // Use an unbounded channel to send requests to the main loop
    let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();

    // Monitor for incoming requests of a daily stoic
    waku_set_event_callback(move |signal| {
        if let waku_bindings::Event::WakuMessage(event) = signal.event() {
            // If not a daily stoic request, return
            if event.waku_message().content_topic() != &DAILY_STOIC_REQUEST_CONTENT_TOPIC {
                return;
            }

            match <DailyStoicRequest as Message>::decode(event.waku_message().payload()) {
                Ok(req) => {
                    // send the request to the channel
                    sender.send(req).expect("ops, couldn't send message");
                }
                Err(e) => {
                    let mut out = std::io::stderr();
                    write!(out, "{e:?}").unwrap();
                    println!();
                }
            }
        }
    });

    // Check if there are enough peers. If not, exit the program
    if app.node_handle.relay_enough_peers(None).is_err() {
        println!("Not enough peers");
        return;
    }

    // Spawn a task that publishes a daily stoic every 24 hours
    let app_timer = app.clone();
    tokio::spawn(async move {
        loop {
            // Publish a random stoic quote
            let quote = app_timer
                .quotes
                .get(rand::random::<usize>() % app_timer.quotes.len())
                .unwrap();
            let stoic = DailyStoic::new(&quote.author, quote.quote.clone());
            publish_daily_stoic(&app_timer.node_handle, stoic).unwrap();

            // Wait for 24 hours
            tokio::time::sleep(std::time::Duration::from_secs(24 * 60 * 60)).await;
        }
    });

    // Wait for incoming requests
    while let Some(_message) = receiver.recv().await {
        // Publish a random stoic quote
        let quote = app
            .quotes
            .get(rand::random::<usize>() % app.quotes.len())
            .unwrap();
        let stoic = DailyStoic::new(&quote.author, quote.quote.clone());
        publish_daily_stoic(&app.node_handle, stoic).unwrap();
    }
}

/// A function that reads quotes from a file
/// # Arguments
/// * `path` - The path to the file containing quotes
/// # Returns
/// The quotes
/// # Errors
/// If the file does not exist or if the quotes are not in JSON format
/// # Examples
/// ```
/// let quotes = read_quotes_from_file("quotes.json").unwrap();
/// ```
fn read_quotes_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<Quote>, Box<dyn Error>> {
    let quotes = std::fs::read_to_string(path)?;
    let quotes: Vec<Quote> = serde_json::from_str(&quotes)?;
    Ok(quotes)
}

/// A function that publishes a daily stoic message to waku
/// # Arguments
/// * `node_handle` - The waku node handle
/// * `stoic` - The daily stoic message
/// # Returns
/// The result of the publish operation
fn publish_daily_stoic(
    node_handle: &WakuNodeHandle<Running>,
    stoic: DailyStoic,
) -> Result<(), Box<dyn Error>> {
    let mut stoic_bytes = Vec::new();
    stoic.encode(&mut stoic_bytes).unwrap();

    // Message to publish with timestamp set to now
    let waku_message = WakuMessage::new(
        stoic_bytes,
        DAILY_STOIC_CONTENT_TOPIC.clone(),
        2,
        (Utc::now().timestamp() as u64).try_into().unwrap(),
    );

    let res = node_handle
        .relay_publish_message(&waku_message, None, None)
        .unwrap();
    println!("Publish result: {res:#?}");
    Ok(())
}
