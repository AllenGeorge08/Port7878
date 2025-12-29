use std::{sync::{Arc, Mutex, mpsc}, thread};

//We’ll also change Job from a struct to a type alias for a trait object that holds the type of closure that execute receive
type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

pub struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>, //q what does a joinHandle do..
}

//e In Production if the OS System doesn't have enough resources, thread::spawn() will panic...
impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.

    pub fn new(size: usize) -> ThreadPool {
        //usize because negative number of threads don't make any sense..
        assert!(size > 0);

        let (sender,receiver) = mpsc::channel();

        //e Arc type will let multiple worker instances own the receiver, and mutex will ensure that only one workers gets a job from the receiver at a time..
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for i in 0..size {
            workers.push(Worker::new(i,Arc::clone(&receiver)));
        }

        ThreadPool { workers,sender: Some(sender)}
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();  //e Unwrapping the option to mpsc::sender,before using send
    }
}

impl Worker {
    fn new(id: usize,receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop{
                let message = receiver.lock().unwrap().recv();

                match message {
                    Ok(job) => {
                        println!("Worker {id} got a job; executing.");
                        job()
                    }
                    Err(_) => {
                        println!("Worker {id} disconnected; shutting down.");
                        break;
                    }
                }
            }
        });

        Worker { id, thread: Some(thread), }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers{
            println!("Shutting down worker: {}", worker.id);

            if let Some(thread) = worker.thread.take(){
                thread.join().unwrap();
            }
         
        }
    }
}