use std::thread;
use std::sync::{Arc, Mutex, Condvar};
use std::time;

pub struct Workers {
    num_workers: usize,
    num_active_workers: usize,
    threads: std::vec::Vec<thread::JoinHandle<()>>,
    pair: Arc<(std::sync::Mutex<JobList>, std::sync::Condvar)>
}

type Job = Box<dyn FnOnce() + Send + 'static>;

struct JobList {
    available: bool,
    jobs: Vec<Job>,
    stopped: bool
}

impl Workers {
    pub fn new(num_workers: usize) -> Self {
        let mut threads = Vec::with_capacity(num_workers);
        let job_list = JobList {
            available: false,
            jobs: Vec::new(),
            stopped: false
        };

        let pair = Arc::new((Mutex::new(job_list), Condvar::new()));

        for _ in 0..(num_workers) {
            let pair_clone = pair.clone();
            threads.push(thread::spawn(move || loop {
                let (lock, condvar) = &*pair_clone;
                let mut job_list = lock.lock().unwrap();
                job_list = condvar.wait_while(job_list, |job_list| !(*job_list).available).unwrap();

                if (*job_list).jobs.len() == 0 && (*job_list).stopped {
                    break;
                }
                
                let job = (*job_list).jobs.pop();

                drop(job_list);
                condvar.notify_one();
                
                if !job.is_none() {
                    (job.unwrap())();
                }
            }));
        }

        Workers {
            num_workers: num_workers,
            num_active_workers: 0,
            threads: threads,
            pair: pair
        }
    }

    pub fn start(&self) {
        let (lock, condvar) = &*self.pair;
        let mut job_list = lock.lock().unwrap();
        
        (*job_list).available = true;
        condvar.notify_one();
    }

    pub fn post<F>(&self, f: F)
        where F: FnOnce() + Send + 'static
    {
        let (lock, condvar) = &*self.pair;
        let mut job_list = lock.lock().unwrap();
        job_list = condvar.wait_while(job_list, |job_list| !(*job_list).available).unwrap();

        let job = Box::new(f);
        (*job_list).jobs.push(job);
    }
    
    pub fn join(self) {
        let (lock, condvar) = &*self.pair;
        let mut job_list = lock.lock().unwrap();
        job_list = condvar.wait_while(job_list, |job_list| !(*job_list).available).unwrap();

        (*job_list).stopped = true;

        drop(job_list);

        self.threads.into_iter().for_each(|thread| {
            thread.join();
        });
    }

    pub fn post_timeout<F>(&self, f: F, timeout: u64)
        where F: FnOnce() + Send + 'static
    {
        self.post(move || {
            println!("Timeout started");
            thread::sleep(time::Duration::from_millis(timeout));
            f();
        });
    }
}