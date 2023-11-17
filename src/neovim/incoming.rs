use crate::rpc;
use std::{collections::BinaryHeap, sync::mpsc};

// NOTE: Responses must be given in reverse order of requests (like "unwinding a stack").

#[derive(Debug, Clone, Default)]
pub struct Incoming {
    requests: Vec<u64>,
    responses: BinaryHeap<QueuedResponse>,
}

impl Incoming {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_request(&mut self, msgid: u64) {
        self.requests.push(msgid);
    }

    pub fn push_response(&mut self, response: rpc::Response, tx: &mpsc::Sender<rpc::Message>) {
        self.responses.push(response.into());
        while let Some(ready) = self.next_ready() {
            tx.send(ready.into()).unwrap();
        }
    }

    fn next_ready(&mut self) -> Option<rpc::Response> {
        if let (Some(id), Some(response)) = (self.requests.last(), self.responses.peek()) {
            if *id == response.0.msgid {
                self.requests.pop();
                self.responses.pop().map(|response| response.into())
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
struct QueuedResponse(rpc::Response);

impl PartialEq for QueuedResponse {
    fn eq(&self, other: &Self) -> bool {
        self.0.msgid == other.0.msgid
    }
}

impl Eq for QueuedResponse {}

impl Ord for QueuedResponse {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.msgid.cmp(&other.0.msgid)
    }
}

impl PartialOrd for QueuedResponse {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl From<rpc::Response> for QueuedResponse {
    fn from(response: rpc::Response) -> Self {
        Self(response)
    }
}

impl From<QueuedResponse> for rpc::Response {
    fn from(value: QueuedResponse) -> Self {
        value.0
    }
}
