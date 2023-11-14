use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use crate::message::*;

pub trait MessageChannel {
    fn push_msg(&mut self, msg: Message);
    fn get_msg(&mut self) -> Option<Message>;
}

pub struct InAndOutMessageChannel {
    queue: VecDeque<Message>,
}

impl InAndOutMessageChannel {
    pub fn new() -> InAndOutMessageChannel {
        InAndOutMessageChannel {
            queue: VecDeque::new(),
        }
    }
}

impl MessageChannel for InAndOutMessageChannel {
    fn push_msg(&mut self, msg: Message) {
        info!("[channel] Message received");
        self.queue.push_back(msg);
    }

    fn get_msg(&mut self) -> Option<Message> {
        self.queue.pop_front()
    }
}

pub struct TransformerListChannel {
    pending: Arc<Mutex<VecDeque<Message>>>,
    done: Arc<Mutex<VecDeque<Message>>>,
}

impl TransformerListChannel {
    pub fn new(
        pending: Arc<Mutex<VecDeque<Message>>>,
        done: Arc<Mutex<VecDeque<Message>>>,
    ) -> TransformerListChannel {
        TransformerListChannel { pending, done }
    }
}

impl MessageChannel for TransformerListChannel {
    fn push_msg(&mut self, msg: Message) {
        info!("[transformer channel] Message received, sending for transformation");
        self.pending.lock().unwrap().push_back(msg);
    }

    fn get_msg(&mut self) -> Option<Message> {
        self.done.lock().expect("Can lock queue").pop_front()
    }
}