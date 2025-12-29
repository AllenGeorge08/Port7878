use std::{sync::{Arc, Mutex, mpsc}, thread};

//We’ll also change Job from a struct to a type alias for a trait object that holds the type of closure that execute receive
type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

pub struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>, //q what does a joinHandle do..
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

        ThreadPool { workers,sender}
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}

impl Worker {
    fn new(id: usize,receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop{
                let job = receiver.lock().unwrap().recv().unwrap(); //e Acquiring a lock might fail if the mutex is in a poisoned state, whihc can happen if some other thread panickedwhile holding the lock rather than releasing the lock..
                //e calling unwrap to have thiss thread panic is the correct action to take...
                println!("Worker {id} got a job; executing.");

                job();
            }
        });

        Worker { id, thread }
    }
}
