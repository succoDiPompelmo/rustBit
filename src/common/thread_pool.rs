use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;

use std::sync::Arc;
use std::sync::Mutex;

use log::error;

use crate::peer::manager::PeerManagerError;
use crate::torrent::writer::WriterError;

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum ThreadPoolError {
    #[error(transparent)]
    PeerManager(#[from] PeerManagerError),
    #[error(transparent)]
    Writer(#[from] WriterError),
}

#[derive(Clone)]
pub struct ThreadPool {
    #[allow(dead_code)]
    workers: Vec<Worker>,
    sender: Sender<Job>,
}

trait FnBox {
    fn call_box(self: Box<Self>) -> Result<(), ThreadPoolError>;
}

impl<F: FnOnce() -> Result<(), ThreadPoolError>> FnBox for F {
    fn call_box(self: Box<F>) -> Result<(), ThreadPoolError> {
        (*self)()
    }
}

type Job = Box<dyn FnBox + Send + 'static>;

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)))
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() -> Result<(), ThreadPoolError> + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(job).unwrap();
    }
}

#[derive(Clone)]
#[allow(dead_code)]
struct Worker {
    id: usize,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv().unwrap();

            match job.call_box() {
                Ok(_) => (),
                Err(err) => error!(
                    "Worker {:?} got an error during job execution: {:?}",
                    id,
                    err.to_string()
                ),
            }
        });

        Worker { id }
    }
}
