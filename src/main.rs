use verust::{
    cown,
    executor::{self, Executor},
    timerfuture,
};

use std::time::Duration;

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
    println!("Starting Verona runtime");
    let exec = executor::Executor::new(4);

    // Test basic cown creation
    println!("Testing cown creation...");
    let counter = cown::Cown::new(42i32);
    println!("Created cown with value 42");

    // Clone the cown to verify reference counting works
    let counter2 = counter.clone();
    println!("Cloned cown");

    // Drop them to verify cleanup
    drop(counter);
    drop(counter2);
    println!("Dropped cowns successfully");

    // Existing async test
    Executor::spawn(say_hello(exec.clone()));

    println!("Running Verona scheduler...");
    exec.run();
}
