use std::{
    sync::{mpsc::Receiver, Arc},
    time::Duration,
};

use crate::endpoint::*;
use crate::message::*;
use crate::non_block_deque::*;
use crate::transformer::*;

pub struct TransformerWorker {
    pending: Arc<NonBlockDeque<Message>>,
    done: Arc<NonBlockDeque<Message>>,
    transformers: Vec<Box<dyn MessageTransformer + Send>>,
    rcv: Receiver<MessageEndpointSignal>,
}

impl TransformerWorker {
    pub fn new(
        transformers: Vec<Box<dyn MessageTransformer + Send>>,
        rcv: Receiver<MessageEndpointSignal>,
        pending: Arc<NonBlockDeque<Message>>,
        done: Arc<NonBlockDeque<Message>>,
    ) -> TransformerWorker {
        TransformerWorker {
            pending,
            done,
            transformers,
            rcv,
        }
    }

    pub fn work_loop(&mut self) {
        loop {
            if let Some(mut msg) = self.pending.pop() {
                for transformer in &mut self.transformers {
                    msg = transformer.transform(msg);
                }

                self.done.push(msg);
            }

            match self.rcv.try_recv() {
                Ok(MessageEndpointSignal::Quit) => break,
                _ => {}
            };
        }
    }
}
