use std::thread;

struct Workers {
    threads: Vec<thread>
}

trait Workers {
    fn start(&self);
    fn post(&self, fn);
    fn join(&self);
}