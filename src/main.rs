use async_trait::async_trait;

use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    rc::Rc,
};

/**
 * MESSAGE LAND
 */

#[async_trait]
trait MessageChannel {
    fn push_msg(&mut self, msg: Message);
    async fn get_message(&mut self) -> Message;
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

#[async_trait]
impl MessageChannel for InAndOutMessageChannel {
    fn push_msg(&mut self, msg: Message) {
        self.queue.push_back(msg);
    }

    async fn get_message(&mut self) -> Message {
        unimplemented!()
    }
}

#[derive(Debug)]
enum MessageTarget {
    Kind(String),
    Id(String),
}

#[derive(Debug)]
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
    kindReceivers: HashMap<String, Rc<RefCell<dyn MessageReceiver>>>,
    idReceivers: HashMap<String, Rc<RefCell<dyn MessageReceiver>>>,
}

impl MessageEndpoint {
    fn new() -> MessageEndpoint {
        MessageEndpoint {
            channel: InAndOutMessageChannel::new(),
            kindReceivers: HashMap::new(),
            idReceivers: HashMap::new(),
        }
    }

    fn send(&mut self, msg: Message) {
        unimplemented!()
    }

    fn set_target(&mut self, target: MessageTarget, receiver: Rc<RefCell<dyn MessageReceiver>>) {
        match target {
            MessageTarget::Id(id) => {
                self.idReceivers.insert(id, receiver);
            }
            MessageTarget::Kind(kind) => {
                self.kindReceivers.insert(kind, receiver);
            }
        }
    }
}

/**
 * USER LAND
 */

struct UserController {
    msg_endpoint: MessageEndpoint,
}

impl UserController {
    fn new(msg_endpoint: MessageEndpoint) -> UserController {
        UserController { msg_endpoint }
    }

    fn save_user(&mut self) {
        self.msg_endpoint.send(Message::new(
            vec![0, 1, 2, 3, 4],
            MessageTarget::Kind("report".into()),
        ));
    }
}

struct Reporter {
    msg_endpoint: MessageEndpoint,
}

impl Reporter {
    fn new(msg_endpoint: MessageEndpoint) -> Rc<RefCell<Reporter>> {
        let reporter = Rc::new(RefCell::new(Reporter { msg_endpoint }));
        reporter
            .borrow_mut()
            .msg_endpoint
            .set_target(MessageTarget::Kind("report".into()), reporter.clone());
        reporter
    }

    fn make_report() {}
}

impl MessageReceiver for Reporter {
    fn on_message(&mut self, msg: Message) {
        println!("Reporter got a message: {:#?}", msg);
    }
}

/**
 * INIT LAND
 */

fn main() {}
