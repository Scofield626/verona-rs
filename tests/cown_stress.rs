use std::sync::{Arc, Barrier};
use std::thread;
use verust::cown::Cown;

#[test]
fn test_cown_creation_stress() {
    // Create many cowns rapidly
    let mut cowns = Vec::new();

    for i in 0..1000 {
        cowns.push(Cown::new(i));
    }

    // Drop them all
    drop(cowns);

    assert!(true);
}

#[test]
fn test_cown_clone_stress() {
    let original = Cown::new(42i32);
    let mut clones = Vec::new();

    // Create many clones
    for _ in 0..100 {
        clones.push(original.clone());
    }

    // Drop all clones
    drop(clones);
    drop(original);

    assert!(true);
}

#[test]
fn test_cown_threaded_creation() {
    let barrier = Arc::new(Barrier::new(4));
    let mut handles = vec![];

    for thread_id in 0..4 {
        let b = barrier.clone();
        let handle = thread::spawn(move || {
            // Wait for all threads to be ready
            b.wait();

            // Each thread creates cowns
            let mut cowns = Vec::new();
            for i in 0..100 {
                cowns.push(Cown::new(thread_id * 1000 + i));
            }

            // Return them to ensure they're dropped in the thread
            cowns
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    assert!(true);
}

#[test]
fn test_cown_threaded_clone() {
    let original = Arc::new(Cown::new(String::from("shared data")));
    let barrier = Arc::new(Barrier::new(4));
    let mut handles = vec![];

    for _ in 0..4 {
        let cown = original.clone();
        let b = barrier.clone();

        let handle = thread::spawn(move || {
            b.wait();

            // Each thread clones the cown multiple times
            let mut clones = Vec::new();
            for _ in 0..50 {
                clones.push((*cown).clone());
            }

            clones
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    assert!(true);
}

#[test]
fn test_cown_interleaved_create_drop() {
    let mut handles = vec![];

    for thread_id in 0..4 {
        let handle = thread::spawn(move || {
            for i in 0..100 {
                let cown = Cown::new(thread_id * 100 + i);

                // Clone it
                let clone = cown.clone();

                // Drop one immediately
                drop(cown);

                // Clone again
                let _clone2 = clone.clone();

                // Drop all
                drop(clone);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    assert!(true);
}

#[test]
fn test_cown_with_large_types_stress() {
    #[derive(Clone)]
    struct LargeStruct {
        data: Vec<u8>,
        strings: Vec<String>,
    }

    let mut cowns = Vec::new();

    for i in 0..100 {
        let large = LargeStruct {
            data: vec![i as u8; 1024], // 1KB each
            strings: vec![format!("string_{}", i); 10],
        };
        cowns.push(Cown::new(large));
    }

    // Clone all
    let clones: Vec<_> = cowns.iter().map(|c| c.clone()).collect();

    drop(cowns);
    drop(clones);

    assert!(true);
}

#[test]
fn test_cown_mixed_types_stress() {
    // Create cowns of many different types simultaneously
    let mut cowns: Vec<Box<dyn std::any::Any>> = Vec::new();

    for i in 0..100 {
        if i % 4 == 0 {
            cowns.push(Box::new(Cown::new(i as i32)));
        } else if i % 4 == 1 {
            cowns.push(Box::new(Cown::new(format!("string_{}", i))));
        } else if i % 4 == 2 {
            cowns.push(Box::new(Cown::new(vec![i; 10])));
        } else {
            cowns.push(Box::new(Cown::new((i as f64, i as f64 * 2.0))));
        }
    }

    drop(cowns);
    assert!(true);
}

#[test]
#[ignore] // Ignore by default as it's slow
fn test_cown_long_running_stress() {
    use std::time::Duration;

    let start = std::time::Instant::now();
    let duration = Duration::from_secs(5);

    let mut iteration = 0;
    while start.elapsed() < duration {
        let mut cowns = Vec::new();
        for i in 0..1000 {
            cowns.push(Cown::new(i));
        }

        // Clone randomly
        for i in 0..500 {
            if i % 2 == 0 {
                let _ = cowns[i].clone();
            }
        }

        drop(cowns);
        iteration += 1;
    }

    println!("Completed {} iterations in 5 seconds", iteration);
    assert!(iteration > 0);
}

#[test]
fn test_cown_drop_in_different_threads() {
    let cown = Arc::new(Cown::new(vec![1, 2, 3, 4, 5]));
    let mut handles = vec![];

    for _ in 0..4 {
        let c = cown.clone();
        let handle = thread::spawn(move || {
            // Clone within thread
            let local_clone = (*c).clone();
            // Let it drop here
            drop(local_clone);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    assert!(true);
}
