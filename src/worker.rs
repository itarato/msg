use std::{
    collections::VecDeque,
    sync::{mpsc::Receiver, Arc, Mutex},
    time::Duration,
};

use crate::endpoint::*;
use crate::message::*;
use crate::transformer::*;

pub struct TransformerWorker {
    pending: Arc<Mutex<VecDeque<Message>>>,
    done: Arc<Mutex<VecDeque<Message>>>,
    transformers: Vec<Box<dyn MessageTransformer + Send>>,
    rcv: Receiver<MessageEndpointSignal>,
}

impl TransformerWorker {
    pub fn new(
        transformers: Vec<Box<dyn MessageTransformer + Send>>,
        rcv: Receiver<MessageEndpointSignal>,
        pending: Arc<Mutex<VecDeque<Message>>>,
        done: Arc<Mutex<VecDeque<Message>>>,
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
            // TODO: MAKE THIS A NON-BUSY LOOP
            if let Some(mut msg) = self.pending.lock().unwrap().pop_front() {
                for transformer in &mut self.transformers {
                    msg = transformer.transform(msg);
                }

                {
                    self.done
                        .lock()
                        .expect("Can lock done queue")
                        .push_back(msg);
                }
            }

            match self.rcv.recv_timeout(Duration::from_millis(10)) {
                Ok(MessageEndpointSignal::Quit) => break,
                _ => {}
            };
        }
    }
}
