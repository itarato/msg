use crate::message::*;

pub trait MessageTransformer {
    fn transform(&mut self, msg: Message) -> Message;
}

pub struct MessageEncryptorTransformer;

impl MessageEncryptorTransformer {
    pub fn new() -> MessageEncryptorTransformer {
        MessageEncryptorTransformer
    }
}

impl MessageTransformer for MessageEncryptorTransformer {
    fn transform(&mut self, msg: Message) -> Message {
        let new_content = msg.content.iter().map(|c| c ^ 0xb0).collect();
        Message::new(new_content, msg.target)
    }
}
