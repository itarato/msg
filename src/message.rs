#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MessageTarget {
    Kind(String),
    Id(String),
}

#[derive(Debug, Clone)]
pub struct Message {
    pub content: Vec<u8>,
    pub target: MessageTarget,
}

impl Message {
    pub fn new(content: Vec<u8>, target: MessageTarget) -> Message {
        Message { content, target }
    }
}
