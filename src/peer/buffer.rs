use crate::messages::Message;
use crate::peer::Peer;

pub struct MessageBuffer<F>
where
    F: FnMut(&mut Peer, usize),
{
    message_filter: fn(&Message) -> bool,
    next_message: F,
    buffer: Vec<Message>,
    capacity: usize,
    expected_size: usize,
}

impl<F> MessageBuffer<F>
where
    F: FnMut(&mut Peer, usize),
{
    pub fn new(
        message_filter: fn(&Message) -> bool,
        next_message: F,
        capacity: usize,
    ) -> MessageBuffer<F> {
        MessageBuffer {
            message_filter,
            next_message,
            buffer: vec![],
            capacity,
            expected_size: 0,
        }
    }

    pub fn push_message(&mut self, msg: Message) {
        if (self.message_filter)(&msg) {
            self.buffer.push(msg);
        }
    }

    pub fn request_next_message(&mut self, peer: &mut Peer) {
        if self.is_len_as_expected() || self.is_full() {
            return;
        }
        (self.next_message)(peer, self.buffer.len());
        self.expected_size += 1;
    }

    pub fn is_full(&mut self) -> bool {
        self.buffer.len() == self.capacity
    }

    pub fn assemble_content(&mut self) -> Vec<u8> {
        self.buffer
            .iter()
            .flat_map(|el| el.get_content_data())
            .collect()
    }

    fn is_len_as_expected(&self) -> bool {
        self.expected_size == (self.buffer.len() + 1)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {}
}
