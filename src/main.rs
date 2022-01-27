mod workers;

fn main() {
    let workers = workers::Workers::new(1);

    workers.post(|| {
        println!("Regular post");
    });

    workers.post_timeout(|| {
        println!("Post timeout 3s")
    }, 3000);

    workers.join();
}
