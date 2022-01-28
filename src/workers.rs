use std::thread;
use std::sync::{Arc, Mutex, Condvar};
use std::time;

pub struct Workers {
    num_workers: usize,
    num_active_workers: usize,
    threads: std::vec::Vec<thread::JoinHandle<(Mutex<bool>, Condvar)>>,
    pair: Arc<(std::sync::Mutex<JobList>, std::sync::Condvar)>
}

struct JobList {
    available: bool,
    jobs: Vec<Job>
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl Workers {
    pub fn new(num_workers: usize) -> Self {
        let mut threads = Vec::with_capacity(num_workers);
        let job_list = JobList {
            available: true,
            jobs: Vec::new()
        };

        let pair = Arc::new((Mutex::new(job_list), Condvar::new()));
        
        for _ in 0..(num_workers) {
            let pair_clone = pair.clone();
            threads.push(thread::spawn(move || loop {
                let (lock, condvar) = &*pair_clone;
                let mut job_list = lock.lock().unwrap();
                job_list = condvar.wait_while(job_list, |job_list| !(*job_list).available).unwrap();

                let job = (*job_list).jobs.pop();

                if !job.is_none() {
                    (job.unwrap())();
                }

                condvar.notify_all();
            }));
        }
        
        let (lock, condvar) = &*pair;
        condvar.notify_all();

        Workers {
            num_workers: num_workers,
            num_active_workers: 0,
            threads: threads,
            pair: pair
        }
    }

    pub fn start(&self) {
        
    }

    pub fn post<F>(&self, f: F)
        where F: FnOnce() + Send + 'static
    {
        let (lock, condvar) = &*self.pair;
        let mut job_list = lock.lock().unwrap();
        while !(*job_list).available {
            job_list = condvar.wait(job_list).unwrap();
        }

        let job = Box::new(f);
        (*job_list).jobs.push(job);
    }
    
    pub fn join(self) {
        self.threads.into_iter().for_each(|thread| {
            thread.join();
        });
    }

    pub fn post_timeout<F>(&self, f: F, timeout: u64)
        where F: FnOnce() + Send + 'static
    {
        self.post(move || {
            thread::sleep(time::Duration::from_millis(timeout));
            f();
        });
    }

    pub fn stop(&self) {

    }
}