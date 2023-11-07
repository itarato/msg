extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use std::{
    collections::{HashMap, VecDeque},
    sync::{
        mpsc::{self, Receiver},
        Arc, Mutex,
    },
    thread,
    time::{self, Duration},
};

/**
 * MESSAGE LAND
 */

trait MessageChannel {
    fn push_msg(&mut self, msg: Message);
    fn get_msg(&mut self) -> Option<Message>;
}

struct InAndOutMessageChannel {
    queue: VecDeque<Message>,
}

impl InAndOutMessageChannel {
    fn new() -> InAndOutMessageChannel {
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

struct TransformerListChannel {
    transformers: Vec<Box<dyn MessageTransformer + Send>>,
    queue: VecDeque<Message>,
}

impl TransformerListChannel {
    fn new(transformers: Vec<Box<dyn MessageTransformer + Send>>) -> TransformerListChannel {
        TransformerListChannel {
            transformers,
            queue: VecDeque::new(),
        }
    }
}

impl MessageChannel for TransformerListChannel {
    fn push_msg(&mut self, msg: Message) {
        info!("[transformer channel] Message received");
        self.queue.push_back(msg);
    }

    fn get_msg(&mut self) -> Option<Message> {
        match self.queue.pop_front() {
            Some(msg) => {
                let mut new_msg = msg;

                // TODO: MAKE THIS ASYNC!!!
                for transformer in &mut self.transformers {
                    new_msg = transformer.transform(new_msg);
                }
                Some(new_msg)
            }
            None => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum MessageTarget {
    Kind(String),
    Id(String),
}

#[derive(Debug, Clone)]
struct Message {
    content: Vec<u8>,
    target: MessageTarget,
}

impl Message {
    fn new(content: Vec<u8>, target: MessageTarget) -> Message {
        Message { content, target }
    }
}

trait MessageTransformer {
    fn transform(&mut self, msg: Message) -> Message;
}

struct MessageEncryptorTransformer;

impl MessageEncryptorTransformer {
    fn new() -> MessageEncryptorTransformer {
        MessageEncryptorTransformer
    }
}

impl MessageTransformer for MessageEncryptorTransformer {
    fn transform(&mut self, msg: Message) -> Message {
        let new_content = msg.content.iter().map(|c| c ^ 0xb0).collect();
        Message::new(new_content, msg.target)
    }
}

trait MessageReceiver {
    fn on_message(&mut self, msg: Message);
}

enum MessageEndpointSignal {
    Quit,
}

struct MessageEndpoint {
    channel: Box<dyn MessageChannel + Send>,
    receivers: HashMap<MessageTarget, Vec<Arc<Mutex<dyn MessageReceiver + Send>>>>,
    rcv: Receiver<MessageEndpointSignal>,
}

impl MessageEndpoint {
    fn new(
        channel: Box<dyn MessageChannel + Send>,
        rcv: Receiver<MessageEndpointSignal>,
    ) -> MessageEndpoint {
        MessageEndpoint {
            channel,
            receivers: HashMap::new(),
            rcv,
        }
    }

    fn send(&mut self, msg: Message) {
        info!("[endpoint] new message arrived");
        self.channel.push_msg(msg);
    }

    fn set_target(
        &mut self,
        target: MessageTarget,
        receiver: Arc<Mutex<dyn MessageReceiver + Send>>,
    ) {
        self.receivers
            .entry(target)
            .or_insert(vec![])
            .push(receiver);
    }

    fn loop_thread(&mut self) {
        loop {
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

/**
 * USER LAND
 */

struct UserController {
    msg_endpoint: Arc<Mutex<MessageEndpoint>>,
}

impl UserController {
    fn new(msg_endpoint: Arc<Mutex<MessageEndpoint>>) -> UserController {
        UserController { msg_endpoint }
    }

    fn save_user(&mut self) {
        self.msg_endpoint.lock().unwrap().send(Message::new(
            vec![0, 1, 2, 3, 4],
            MessageTarget::Kind("report".into()),
        ));
    }
}

struct Reporter;

impl Reporter {
    fn new() -> Reporter {
        Reporter {}
    }
}

impl MessageReceiver for Reporter {
    fn on_message(&mut self, msg: Message) {
        info!("Message arrived in user land");

        println!("Reporter got a message: {:#?}", msg);
    }
}

/**
 * INIT LAND
 */

fn main() {
    pretty_env_logger::init();

    info!("Experiment start");

    info!("Setting up messaging components");

    let (snd, rcv) = mpsc::channel();
    let channel = TransformerListChannel::new(vec![Box::new(MessageEncryptorTransformer::new())]);
    // let channel = InAndOutMessageChannel::new();
    let msg_endpoint = Arc::new(Mutex::new(MessageEndpoint::new(Box::new(channel), rcv)));

    let thread_msg_endpoint = msg_endpoint.clone();
    let msg_loop = thread::spawn(move || {
        thread_msg_endpoint.lock().unwrap().loop_thread();
    });

    // ACTION START

    info!("Setting up interactors");

    let mut user_ctrl = UserController::new(msg_endpoint.clone());

    let reporter1 = Reporter::new();
    let reporter2 = Reporter::new();

    msg_endpoint.lock().unwrap().set_target(
        MessageTarget::Kind("report".into()),
        Arc::new(Mutex::new(reporter1)),
    );
    msg_endpoint.lock().unwrap().set_target(
        MessageTarget::Kind("report".into()),
        Arc::new(Mutex::new(reporter2)),
    );

    info!("Trigger message creation");

    user_ctrl.save_user();

    info!("Artificial sleep");
    thread::sleep(time::Duration::from_millis(10));

    // ACTION END

    info!("Send QUIT command to message endpoint");
    snd.send(MessageEndpointSignal::Quit)
        .expect("mpsc command sent");

    info!("Wait for message loop thread to finish");
    msg_loop.join().expect("msg loop thread joins");
}
