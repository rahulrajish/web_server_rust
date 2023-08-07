use std::{
    sync::{mpsc::{self}, Arc, Mutex}, 
    thread,
};

pub struct ThreadPool{
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;


/// Create a new ThreadPool
/// 
/// The size is the number of threads in the pool
/// 
/// # Panics
/// 
/// The `new` function will panic if the size is zero

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { 
            workers, 
            sender: Some(sender), 
        }
    }
}

/// Adding a request to the thread
/// 
/// Accepts closure as input 

impl ThreadPool {
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
        {
            let job = Box::new(f);

            //self.sender.as_ref().unwrap().send(job).unwrap();

            match self.sender.as_ref() {
                Some(sender) =>  {
                    if let Err(_e) = sender.send(job) {
                        println!("Error while sending threads from threadpool");
                    }
                },
                None => println!("Error while receving threads from threadpool"),
            }
        }
}

/// Dropping the threadpool after execution
impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                if let Err(_e) = thread.join() {
                    println!("Error while shutting down thread");
                }
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move|| loop {
            //let message = receiver.lock().unwrap().recv();

            let message = match receiver.lock() {
                Ok(receiver) => receiver.recv(),
                Err(_) => {
                    println!("Error while receiving new Worker Thread");
                    break;
                },
            };

            match message {
                Ok(job) => {
                    println!("Worker {id} got a job; executing.");

                    job();
                },
                Err(_) => {
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
            
        });

        Worker {
            id,
            thread: Some(thread),
        }
    } 
}