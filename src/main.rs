mod protocol;

use std::error::Error;
use std::io::Write;

use crate::protocol::{DailyStoic, DAILY_STOIC_CONTENT_TOPIC};
use chrono::Utc;
use prost::Message;

use protocol::DailyStoicRequest;
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
        node_handle.connect_peer_with_id(peer_id, None)?;
    }

    node_handle.relay_subscribe(None)?;
    Ok(node_handle)
}

fn main() {
    let node_handle = setup_node_handle().unwrap();

    waku_set_event_callback(move |signal| match signal.event() {
        waku_bindings::Event::WakuMessage(event) => {
            match <DailyStoicRequest as Message>::decode(event.waku_message().payload()) {
                Ok(req) => {
                    // send daily stoic
                    unimplemented!();
                }
                Err(e) => {
                    let mut out = std::io::stderr();
                    write!(out, "{e:?}").unwrap();
                }
            }
        }
        waku_bindings::Event::Unrecognized(data) => {
            let mut out = std::io::stderr();
            write!(out, "Error, received unrecognized event {data}").unwrap();
        }
        _ => {}
    });
}

