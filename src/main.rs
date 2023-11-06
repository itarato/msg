use std::{
    collections::{HashMap, VecDeque},
    sync::{
        atomic::{self, AtomicBool},
        Arc, Mutex,
    },
    thread, time,
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
        println!("CHANNEL -> message received");
        self.queue.push_back(msg);
    }

    fn get_msg(&mut self) -> Option<Message> {
        println!("CHANNEL -> message popped");
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

struct MessageEndpoint {
    channel: InAndOutMessageChannel,
    kind_receivers: HashMap<String, Arc<Mutex<dyn MessageReceiver + Send>>>,
    id_receivers: HashMap<String, Arc<Mutex<dyn MessageReceiver + Send>>>,
    is_running: AtomicBool,
}

impl MessageEndpoint {
    fn new() -> MessageEndpoint {
        MessageEndpoint {
            channel: InAndOutMessageChannel::new(),
            kind_receivers: HashMap::new(),
            id_receivers: HashMap::new(),
            is_running: AtomicBool::new(true),
        }
    }

    fn send(&mut self, msg: Message) {
        println!("MSG_ENDPOINT -> enqueue message");
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
        while self.is_running.load(atomic::Ordering::SeqCst) {
            println!(
                "MSG_ENDPOINT -> is running: {}",
                self.is_running.load(atomic::Ordering::SeqCst)
            );

            if let Some(msg) = self.channel.get_msg() {
                println!("MSG_ENDPOINT -> message found");
                match msg.target.clone() {
                    MessageTarget::Id(id) => unimplemented!(),
                    MessageTarget::Kind(kind) => {
                        if let Some(receiver) = self.kind_receivers.get(&kind) {
                            receiver.lock().unwrap().on_message(msg);
                        }
                    }
                }
            }
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
        println!("Reporter got a message: {:#?}", msg);
    }
}

/**
 * INIT LAND
 */

fn main() {
    let msg_endpoint = Arc::new(Mutex::new(MessageEndpoint::new()));

    let thread_msg_endpoint = msg_endpoint.clone();

    let msg_loop = thread::spawn(move || {
        thread_msg_endpoint.lock().unwrap().loop_thread();
    });

    // ACTION START

    let mut user_ctrl = UserController::new(msg_endpoint.clone());
    let mut _reporter = Reporter::new(msg_endpoint.clone());

    user_ctrl.save_user();

    thread::sleep(time::Duration::from_nanos(1));

    // ACTION END

    println!("MAIN -> STOP");
    msg_endpoint.lock().unwrap().stop();
    msg_loop.join().expect("msg loop thread joins");
}
