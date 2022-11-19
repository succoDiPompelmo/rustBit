#[derive(Debug, Clone)]
pub struct InterestedMessage {}

impl InterestedMessage {
    pub fn from_bytes() -> InterestedMessage {
        InterestedMessage {}
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        vec![]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_as_bytes() {
        let message = InterestedMessage {};
        let expect: Vec<u8> = vec![];
        assert_eq!(message.as_bytes(), expect);
    }
}
