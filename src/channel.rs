use std::{collections::VecDeque, sync::Arc};

use crate::message::*;
use crate::non_block_deque::*;

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
    pending: Arc<NonBlockDeque<Message>>,
    done: Arc<NonBlockDeque<Message>>,
}

impl TransformerListChannel {
    pub fn new(
        pending: Arc<NonBlockDeque<Message>>,
        done: Arc<NonBlockDeque<Message>>,
    ) -> TransformerListChannel {
        TransformerListChannel { pending, done }
    }
}

impl MessageChannel for TransformerListChannel {
    fn push_msg(&mut self, msg: Message) {
        info!("[transformer channel] Message received, sending for transformation");
        self.pending.push(msg);
    }

    fn get_msg(&mut self) -> Option<Message> {
        self.done.pop()
    }
}
