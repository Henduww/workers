mod workers;

fn main() {
    let workers = workers::Workers::new(4);

    workers.post(|| {
        println!("Regular post 1");
    });

    workers.post_timeout(|| {
        println!("Post timeout 1 3s")
    }, 3000);

    workers.post(|| {
        println!("Regular post 2");
    });

    workers.post(|| {
        println!("Regular post 3");
    });

    workers.post_timeout(|| {
        println!("Post timeout 2 6s")
    }, 6000);

    workers.post(|| {
        println!("Regular post 4");
    });

    workers.join();
}
