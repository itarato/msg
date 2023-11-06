extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use std::{
    collections::{HashMap, VecDeque},
    sync::{
        atomic::{self, AtomicBool},
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

#[derive(Debug, Clone)]
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

trait MessageReceiver {
    fn on_message(&mut self, msg: Message);
}

enum MessageEndpointSignal {
    Quit,
}

struct MessageEndpoint {
    channel: InAndOutMessageChannel,
    kind_receivers: HashMap<String, Arc<Mutex<dyn MessageReceiver + Send>>>,
    id_receivers: HashMap<String, Arc<Mutex<dyn MessageReceiver + Send>>>,
    is_running: AtomicBool,
    rcv: Receiver<MessageEndpointSignal>,
}

impl MessageEndpoint {
    fn new(rcv: Receiver<MessageEndpointSignal>) -> MessageEndpoint {
        MessageEndpoint {
            channel: InAndOutMessageChannel::new(),
            kind_receivers: HashMap::new(),
            id_receivers: HashMap::new(),
            is_running: AtomicBool::new(true),
            rcv,
        }
    }

    fn send(&mut self, msg: Message) {
        info!("[endpoint] new message arrived");
        self.channel.push_msg(msg);
    }

    fn stop(&mut self) {
        self.is_running.store(false, atomic::Ordering::SeqCst);
    }

    fn set_target(
        &mut self,
        target: MessageTarget,
        receiver: Arc<Mutex<dyn MessageReceiver + Send>>,
    ) {
        match target {
            MessageTarget::Id(id) => {
                self.id_receivers.insert(id, receiver);
            }
            MessageTarget::Kind(kind) => {
                self.kind_receivers.insert(kind, receiver);
            }
        }
    }

    fn loop_thread(&mut self) {
        loop {
            if let Some(msg) = self.channel.get_msg() {
                info!("[endpoint] [loop] new message is being transferred");

                match msg.target.clone() {
                    MessageTarget::Id(id) => unimplemented!(),
                    MessageTarget::Kind(kind) => {
                        if let Some(receiver) = self.kind_receivers.get(&kind) {
                            receiver.lock().unwrap().on_message(msg);
                        }
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
    fn new(msg_endpoint: Arc<Mutex<MessageEndpoint>>) -> Arc<Mutex<Reporter>> {
        let reporter = Arc::new(Mutex::new(Reporter {}));

        msg_endpoint
            .lock()
            .unwrap()
            .set_target(MessageTarget::Kind("report".into()), reporter.clone());

        reporter
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
    let msg_endpoint = Arc::new(Mutex::new(MessageEndpoint::new(rcv)));

    let thread_msg_endpoint = msg_endpoint.clone();
    let msg_loop = thread::spawn(move || {
        thread_msg_endpoint.lock().unwrap().loop_thread();
    });

    // ACTION START

    info!("Setting up interactors");

    let mut user_ctrl = UserController::new(msg_endpoint.clone());
    Reporter::new(msg_endpoint.clone());

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
