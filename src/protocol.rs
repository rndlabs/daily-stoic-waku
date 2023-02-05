use chrono::{DateTime, LocalResult, TimeZone, Utc};
use prost::Message;
use waku_bindings::{Encoding, WakuContentTopic};

pub static DAILY_STOIC_REQUEST_CONTENT_TOPIC: WakuContentTopic = 
    WakuContentTopic::new("dailystoic", 1, "request", Encoding::Proto);

pub static DAILY_STOIC_CONTENT_TOPIC: WakuContentTopic = 
    WakuContentTopic::new("dailystoic", 1, "broadcast", Encoding::Proto);

#[derive(Clone, Message)]
pub struct DailyStoic {
    #[prost(uint64, tag = "1")]
    timestamp: u64,
    #[prost(string, tag = "2")]
    author: String,
    #[prost(bytes, tag = "3")]
    content: Vec<u8>,
}

#[derive(Clone, Message)]
pub struct DailyStoicRequest {
    #[prost(uint64, tag = "1")]
    timestamp: u64
}

impl DailyStoic {
    pub fn new(author: &str, content: String) -> Self {
        Self {
            timestamp: Utc::now().timestamp() as u64,
            author: author.to_string(),
            content: content.as_bytes().to_vec(),
        }
    }

    pub fn content(&self) -> String {
        String::from_utf8(self.content.clone()).unwrap()
    }

    pub fn author(&self) -> &str {
        &self.author
    }

    pub fn timestamp(&self) -> LocalResult<DateTime<Utc>> {
        Utc.timestamp_opt(self.timestamp as i64, 0)
    }

}