use std::thread;

struct Workers {
    threads: Vec<thread>
}

trait Workers {
    fn start(&self);
    fn post(&self, fn);
    fn post_timeout(&self, fn, i32);
    fn join(&self);
    fn stop(&self);
}