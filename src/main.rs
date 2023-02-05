mod protocol;

use std::error::Error;
use std::io::Write;

use crate::protocol::{DailyStoic, DAILY_STOIC_CONTENT_TOPIC};
use chrono::Utc;
use prost::Message;

use protocol::{DailyStoicRequest, DAILY_STOIC_REQUEST_CONTENT_TOPIC};
use url::Url;
use waku_bindings::{
    waku_new, waku_set_event_callback, ContentFilter, Multiaddr, PagingOptions, ProtocolId,
    Running, StoreQuery, WakuMessage, WakuNodeHandle, WakuNodeConfig,
};

pub static ENRTREE: &str = "enrtree://AOGECG2SPND25EEFMAJ5WF3KSGJNSGV356DSTL2YVLLZWIV6SAYBM@prod.waku.nodes.status.im";

fn setup_node_handle() -> Result<WakuNodeHandle<Running>, Box<dyn Error>> {
    let node_handle = waku_new(None)?;
    let node_handle = node_handle.start()?;

    let addresses = node_handle.dns_discovery(&Url::parse(ENRTREE)?, None, None)?;

    for address in addresses {
        let peer_id = node_handle.add_peer(&address, ProtocolId::Relay)?;
        node_handle.connect_peer_with_id(peer_id, None).unwrap_or_else(|e| {
            let mut out = std::io::stderr();
            write!(out, "Error, couldn't connect to peer {e:?}").unwrap();
        });
    }

    node_handle.relay_subscribe(None)?;
    Ok(node_handle)
}

#[tokio::main]
async fn main() {
    let node_handle = setup_node_handle().unwrap();
    let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();

    waku_set_event_callback(move |signal| match signal.event() {
        waku_bindings::Event::WakuMessage(event) => {
            // Check if the message is a daily stoic request

            if event.waku_message().content_topic() != &DAILY_STOIC_REQUEST_CONTENT_TOPIC {
                println!("Not a daily stoic request");
                return;
            }

            println!("Content topic: {:#?}", event.waku_message().content_topic());

            // Print the hex encoded payload
            println!("Payload: {:#?}", hex::encode(event.waku_message().payload()));

            match <DailyStoicRequest as Message>::decode(event.waku_message().payload()) {
                Ok(req) => {
                    // send daily stoic
                    sender.send(req).expect("ups, couldn't send message");
                }
                Err(e) => {
                    let mut out = std::io::stderr();
                    write!(out, "{e:?}").unwrap();
                    println!();
                }
            }
        }
        // waku_bindings::Event::Unrecognized(data) => {
        //     let mut out = std::io::stderr();
        //     write!(out, "Error, received unrecognized event {data}").unwrap();
        //     println!();
        // }
        _ => {}
    });

    // Check if there are enough peers
    loop {
        if node_handle.relay_enough_peers(None).is_ok() {
            break;
        }
        println!("Not enough peers");
        // sleep for 1 second
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

    // Publish a daily stoic
    let stoic = DailyStoic::new("Seneca", "The greatest obstacle to living is expectancy, which hangs upon tomorrow and loses today. You are arranging what lies in Fortune's control, and abandoning what lies in yours. What are you looking at? To what goal are you straining? The whole future lies in uncertainty: live immediately.".to_string());
    let mut stoic_bytes = Vec::new();
    stoic.encode(&mut stoic_bytes).unwrap();

    let waku_message = WakuMessage::new(
        stoic_bytes,
        DAILY_STOIC_CONTENT_TOPIC.clone(),
        2,
        0,
    );

    let res = node_handle.relay_publish_message(&waku_message, None, None).unwrap();
    println!("Publish result: {:#?}", res);

    while let Some(message) = receiver.recv().await {
        println!("Received message: {:#?}", message);

        // Publish a daily stoic
        let stoic = DailyStoic::new("Seneca", "The greatest obstacle to living is expectancy, which hangs upon tomorrow and loses today. You are arranging what lies in Fortune's control, and abandoning what lies in yours. What are you looking at? To what goal are you straining? The whole future lies in uncertainty: live immediately.".to_string());
        let mut stoic_bytes = Vec::new();
        stoic.encode(&mut stoic_bytes).unwrap();

        let waku_message = WakuMessage::new(
            stoic_bytes,
            DAILY_STOIC_CONTENT_TOPIC.clone(),
            2,
            0,
        );

        let res = node_handle.relay_publish_message(&waku_message, None, None).unwrap();
        println!("Publish result: {:#?}", res);
    }
}

