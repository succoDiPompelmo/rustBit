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
        if (self.message_filter)(&msg) && !self.is_full() {
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
    fn test_push_message_not_filterd() {
        let mut buffer = MessageBuffer::new(|_| true, |_, _| {}, 10);
        let message = Message::new_raw(vec![], 1, 1).unwrap();

        buffer.push_message(message);
        assert_eq!(buffer.buffer.len(), 1);
        assert!(!buffer.is_full());
    }

    #[test]
    fn test_push_message_filterd() {
        let mut buffer = MessageBuffer::new(|_| false, |_, _| {}, 10);
        let message = Message::new_raw(vec![], 1, 1).unwrap();

        buffer.push_message(message);
        assert_eq!(buffer.buffer.len(), 0);
        assert!(!buffer.is_full());
    }

    #[test]
    fn test_push_message_discarded_when_full() {
        let mut buffer = MessageBuffer::new(|_| true, |_, _| {}, 0);
        let message = Message::new_raw(vec![], 1, 1).unwrap();

        buffer.push_message(message);
        assert_eq!(buffer.buffer.len(), 0);
        assert!(buffer.is_full());
    }

    #[test]
    fn test_push_message_mix() {
        let mut buffer = MessageBuffer::new(|msg| msg.get_id() % 2 == 1, |_, _| {}, 2);
        let message_1 = Message::new_raw(vec![], 1, 1).unwrap();
        let message_2 = Message::new_raw(vec![], 1, 2).unwrap();
        let message_3 = Message::new_raw(vec![], 1, 3).unwrap();
        let message_4 = Message::new_raw(vec![], 1, 5).unwrap();

        buffer.push_message(message_1);
        buffer.push_message(message_2);
        buffer.push_message(message_3);
        buffer.push_message(message_4);
        assert_eq!(buffer.buffer.len(), 2);
        assert!(buffer.is_full());
    }

    #[test]
    fn test_assemble_content() {
        let mut buffer = MessageBuffer::new(|_| true, |_, _| {}, 2);
        let message_1 = Message::new_raw(vec![0x01], 2, 5).unwrap();
        let message_2 = Message::new_raw(vec![0x02, 0x00], 2, 5).unwrap();

        buffer.push_message(message_1);
        buffer.push_message(message_2);
        let expected = vec![0x01, 0x02, 0x00];
        assert_eq!(buffer.assemble_content(), expected);
        assert!(buffer.is_full());
    }

    #[test]
    fn test_assemble_empty_buffer() {
        let mut buffer = MessageBuffer::new(|_| true, |_, _| {}, 2);

        let expected: Vec<u8> = vec![];
        assert_eq!(buffer.assemble_content(), expected);
        assert!(!buffer.is_full());
    }
}
