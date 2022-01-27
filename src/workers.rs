use std::thread;
use std::sync::{Arc, Mutex, mpsc};

pub struct Workers {
    num_workers: usize,
    num_active_workers: usize,
    threads: Vec<thread::JoinHandle<Arc<Mutex<mpsc::Receiver<()>>>>>,
    sender: mpsc::Sender<Job>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl Workers {
    pub fn new(num_workers: usize) -> Self {
        let mut threads = Vec::with_capacity(num_workers);

        let (sender, receiver) = mpsc::channel();

        let sender: mpsc::Sender<Job> = sender;
        let receiver = Arc::new(Mutex::new(receiver));

        for _ in 0..(num_workers) {
            let thread_receiver = Arc::clone(&receiver);
            threads.push(thread::spawn(move || loop {
                let job = thread_receiver.lock().unwrap().recv().unwrap();

                job();
            }));
        }

        Workers {
            num_workers: num_workers,
            num_active_workers: 0,
            threads: threads,
            sender: sender,
        }
    }

    pub fn start(&self) {
        
    }

    pub fn post<F>(&self, f: F)
        where F: FnOnce() + Send + 'static
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
    
    pub fn join(self) {
        self.threads.into_iter().for_each(|thread| {
            thread.join();
        });
    }

    pub fn post_timeout<F>(&self, f: F, timeout: u32)
        where F: FnOnce() + Send + 'static
    {
        self.post(move || {
            thread::sleep_ms(timeout);
            f();
        });
    }

    pub fn stop(&self) {

    }
}