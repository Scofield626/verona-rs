mod executor;
mod timerfuture;
mod verona_stubs;

use std::time::Duration;

use crate::executor::Executor;

async fn say_hello(exec: Executor) {
    println!("hello");
    timerfuture::TimerFuture::new(Duration::new(2, 0)).await;
    println!("world");

    Executor::spawn(async {
        println!("I can spawn like this too");
    });

    exec.shutdown();
}

fn main() {
    let exec = executor::Executor::new(2);

    Executor::spawn(say_hello(exec));

    exec.run(); // wait for all shutdown signals
}
