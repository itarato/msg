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
    channel: Box<dyn MessageChannel + Send>,
    receivers: HashMap<MessageTarget, Vec<Arc<Mutex<dyn MessageReceiver + Send>>>>,
    rcv: Receiver<MessageEndpointSignal>,
}

impl MessageEndpoint {
    pub fn new(
        channel: Box<dyn MessageChannel + Send>,
        rcv: Receiver<MessageEndpointSignal>,
    ) -> MessageEndpoint {
        MessageEndpoint {
            channel,
            receivers: HashMap::new(),
            rcv,
        }
    }

    pub fn send(&mut self, msg: Message) {
        info!("[endpoint] new message arrived");
        self.channel.push_msg(msg);
    }

    pub fn set_target(
        &mut self,
        target: MessageTarget,
        receiver: Arc<Mutex<dyn MessageReceiver + Send>>,
    ) {
        self.receivers
            .entry(target)
            .or_insert(vec![])
            .push(receiver);
    }

    pub fn loop_thread(&mut self) {
        loop {
            // TODO: MAKE THIS A NOT-BUSY-LOOP
            if let Some(msg) = self.channel.get_msg() {
                info!("[endpoint] [loop] new message is being transferred");
                if let Some(receivers) = self.receivers.get(&msg.target) {
                    for receiver in receivers {
                        receiver.lock().unwrap().on_message(msg.clone());
                    }
                }
            }

            match self.rcv.recv_timeout(Duration::from_nanos(1)) {
                Ok(MessageEndpointSignal::Quit) => break,
                _ => {}
            };
        }
    }
}
