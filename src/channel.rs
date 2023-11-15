use std::sync::Arc;
use std::time::Duration;

use crate::message::*;
use crate::non_block_deque::*;

pub trait MessageChannel {
    fn push_msg(&self, msg: Message);
    fn get_msg(&self) -> Option<Message>;
}

pub struct InAndOutMessageChannel {
    queue: NonBlockDeque<Message>,
}

impl InAndOutMessageChannel {
    pub fn new() -> InAndOutMessageChannel {
        InAndOutMessageChannel {
            queue: NonBlockDeque::new(Duration::from_millis(10)),
        }
    }
}

impl MessageChannel for InAndOutMessageChannel {
    fn push_msg(&self, msg: Message) {
        info!("[channel] Message received");
        self.queue.push(msg);
    }

    fn get_msg(&self) -> Option<Message> {
        self.queue.pop()
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
    fn push_msg(&self, msg: Message) {
        info!("[transformer channel] Message received, sending for transformation");
        self.pending.push(msg);
    }

    fn get_msg(&self) -> Option<Message> {
        self.done.pop()
    }
}
