use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

#[derive(Debug, Clone)]
pub struct PiecePool(Arc<Mutex<VecDeque<usize>>>);

impl PiecePool {
    pub fn new(count: usize) -> Self {
        let mut queue = VecDeque::new();

        for i in 0..count {
            queue.push_back(i);
        }

        PiecePool(Arc::new(Mutex::new(queue)))
    }

    pub fn pop(&self) -> Option<usize> {
        match self.pool().lock() {
            Ok(mut piece_pool) => piece_pool.pop_front(),
            Err(_) => None,
        }
    }

    pub fn insert(&self, piece: usize) {
        if let Ok(mut piece_pool) = self.pool().lock() {
            piece_pool.push_back(piece)
        }
    }

    #[cfg(test)]
    pub fn is_emtpy(&self) -> bool {
        match self.pool().lock() {
            Ok(piece_pool) => piece_pool.is_empty(),
            Err(_) => true,
        }
    }

    fn pool(&self) -> &Arc<Mutex<VecDeque<usize>>> {
        &self.0
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_next() {
        let pool = PiecePool::new(1);

        assert_eq!(pool.pop(), Some(0));
        assert_eq!(pool.pop(), None);
    }

    #[test]
    fn test_empty() {
        let pool = PiecePool::new(0);

        assert!(pool.is_emtpy());
    }

    #[test]
    fn test_insert() {
        let pool = PiecePool::new(0);

        pool.insert(0);
        assert!(!pool.is_emtpy());
    }
}
