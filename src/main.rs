extern crate pretty_env_logger;
#[macro_use]
extern crate log;

mod channel;
mod endpoint;
mod message;
mod transformer;
mod worker;

use crate::channel::*;
use crate::endpoint::*;
use crate::message::*;
use crate::transformer::*;
use crate::worker::*;

use std::{
    collections::VecDeque,
    sync::{
        mpsc::{self},
        Arc, Mutex,
    },
    thread,
    time::{self},
};

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
        info!("[user ctrl] Sending message");
        self.msg_endpoint.lock().unwrap().send(Message::new(
            vec![0, 1, 2, 3, 4],
            MessageTarget::Kind("report".into()),
        ));
    }

    fn fake_mutation(&mut self) {}
}

struct Reporter;

impl Reporter {
    fn new() -> Reporter {
        Reporter {}
    }

    fn fake_mutation(&mut self) {}
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

    let (tf_wrk_snd, tf_wrk_rcv) = mpsc::channel();
    let worker_input_queue = Arc::new(Mutex::new(VecDeque::new()));
    let worker_output_queue = Arc::new(Mutex::new(VecDeque::new()));
    let transform_worker = Arc::new(Mutex::new(TransformerWorker::new(
        vec![Box::new(MessageEncryptorTransformer::new())],
        tf_wrk_rcv,
        worker_input_queue.clone(),
        worker_output_queue.clone(),
    )));

    let transform_worker_thread = thread::spawn({
        let worker = transform_worker.clone();
        move || {
            worker.lock().unwrap().work_loop();
        }
    });

    let (snd, rcv) = mpsc::channel();
    let channel = TransformerListChannel::new(worker_input_queue, worker_output_queue);
    let msg_endpoint = Arc::new(Mutex::new(MessageEndpoint::new(Box::new(channel), rcv)));

    let thread_msg_endpoint = msg_endpoint.clone();

    // ACTION START

    info!("Setting up interactors");

    let mut user_ctrl = UserController::new(msg_endpoint.clone());

    let reporter1 = Arc::new(Mutex::new(Reporter::new()));
    let reporter2 = Reporter::new();

    msg_endpoint
        .lock()
        .unwrap()
        .set_target(MessageTarget::Kind("report".into()), reporter1.clone());
    msg_endpoint.lock().unwrap().set_target(
        MessageTarget::Kind("report".into()),
        Arc::new(Mutex::new(reporter2)),
    );

    user_ctrl.fake_mutation();
    reporter1.lock().unwrap().fake_mutation();

    info!("Trigger message creation");

    user_ctrl.save_user();

    info!("Artificial sleep");

    // ACTION END

    let msg_loop = thread::spawn(move || {
        thread_msg_endpoint.lock().unwrap().loop_thread();
    });

    thread::sleep(time::Duration::from_millis(10));

    info!("Send QUIT command to message endpoint");
    tf_wrk_snd
        .send(MessageEndpointSignal::Quit)
        .expect("worker thread stop command sent");
    snd.send(MessageEndpointSignal::Quit)
        .expect("mpsc command sent");

    info!("Wait for message loop thread to finish");
    msg_loop.join().expect("msg loop thread joins");
    transform_worker_thread
        .join()
        .expect("worker thread joined");
}
