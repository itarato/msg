use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use crate::channel::*;
use crate::message::*;

pub trait MessageReceiver {
    fn on_message(&mut self, msg: Message);
}

pub enum MessageEndpointSignal {
    Quit,
}

pub struct MessageEndpoint {
    channel: Box<dyn MessageChannel + Send + Sync>,
    receivers: Mutex<HashMap<MessageTarget, Vec<Arc<Mutex<dyn MessageReceiver + Send>>>>>,
}

impl MessageEndpoint {
    pub fn new(channel: Box<dyn MessageChannel + Send + Sync>) -> MessageEndpoint {
        MessageEndpoint {
            channel,
            receivers: Mutex::new(HashMap::new()),
        }
    }

    pub fn send(&self, msg: Message) {
        info!("[endpoint] new message arrived");
        self.channel.push_msg(msg);
    }

    pub fn set_target(
        &self,
        target: MessageTarget,
        receiver: Arc<Mutex<dyn MessageReceiver + Send>>,
    ) {
        self.receivers
            .lock()
            .unwrap()
            .entry(target)
            .or_insert(vec![])
            .push(receiver);
    }

    pub fn loop_thread(&self, end_receiver: Receiver<MessageEndpointSignal>) {
        loop {
            if let Some(msg) = self.channel.get_msg() {
                info!("[endpoint] [loop] new message is being transferred");
                if let Some(receivers) = self.receivers.lock().unwrap().get(&msg.target) {
                    for receiver in receivers {
                        receiver.lock().unwrap().on_message(msg.clone());
                    }
                }
            }

            match end_receiver.try_recv() {
                Ok(MessageEndpointSignal::Quit) => break,
                _ => {}
            };
        }
    }
}
