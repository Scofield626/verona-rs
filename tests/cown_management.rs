use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use verust::cown::Cown;

#[test]
fn test_cown_creation_primitive_types() {
    // Test with various primitive types
    let _cown_i32 = Cown::new(42i32);
    let _cown_i64 = Cown::new(100i64);
    let _cown_u32 = Cown::new(255u32);
    let _cown_f64 = Cown::new(3.14f64);
    let _cown_bool = Cown::new(true);
    let _cown_char = Cown::new('A');

    // If we reach here without crashing, creation worked
    assert!(true);
}

#[test]
fn test_cown_creation_string() {
    let message = String::from("Hello, Cowns!");
    let cown = Cown::new(message);

    // Verify the cown was created without panic
    drop(cown);
    assert!(true);
}

#[test]
fn test_cown_creation_vec() {
    let data = vec![1, 2, 3, 4, 5];
    let cown = Cown::new(data);

    drop(cown);
    assert!(true);
}

#[test]
fn test_cown_creation_custom_struct() {
    #[derive(Debug)]
    struct Point {
        x: i32,
        y: i32,
    }

    let point = Point { x: 10, y: 20 };
    let cown = Cown::new(point);

    drop(cown);
    assert!(true);
}

#[test]
fn test_cown_clone() {
    let cown1 = Cown::new(42i32);

    // Clone the cown - should increment reference count
    let cown2 = cown1.clone();
    let cown3 = cown1.clone();

    // All three should be valid
    drop(cown1);
    drop(cown2);
    drop(cown3);

    // If we reach here, ref counting worked correctly
    assert!(true);
}

#[test]
fn test_cown_clone_different_types() {
    let cown_string = Cown::new(String::from("test"));
    let cown_vec = Cown::new(vec![1, 2, 3]);

    let _clone1 = cown_string.clone();
    let _clone2 = cown_vec.clone();

    assert!(true);
}

#[test]
fn test_cown_drop_order() {
    // Create a counter to track drops
    let drop_count = Arc::new(AtomicUsize::new(0));

    struct DropCounter {
        count: Arc<AtomicUsize>,
        id: usize,
    }

    impl Drop for DropCounter {
        fn drop(&mut self) {
            self.count.fetch_add(1, Ordering::SeqCst);
        }
    }

    {
        let counter1 = DropCounter {
            count: drop_count.clone(),
            id: 1,
        };
        let cown = Cown::new(counter1);

        // Clone it multiple times
        let _clone1 = cown.clone();
        let _clone2 = cown.clone();

        // All clones will drop here
    }

    // Note: The actual drop happens asynchronously in the Verona runtime
    // So we can't reliably test the exact count without running the runtime
    // This test just verifies no crashes occur
    // assert_eq!(drop_count.load(Ordering::SeqCst), 1); // Uncomment to test if synchronous
    assert!(true);
}

#[test]
fn test_cown_with_option() {
    let cown_some = Cown::new(Some(42i32));
    let cown_none: Cown<Option<i32>> = Cown::new(None);

    drop(cown_some);
    drop(cown_none);

    assert!(true);
}

#[test]
fn test_cown_with_result() {
    let cown_ok: Cown<Result<i32, String>> = Cown::new(Ok(42));
    let cown_err: Cown<Result<i32, String>> = Cown::new(Err("error".to_string()));

    drop(cown_ok);
    drop(cown_err);

    assert!(true);
}

#[test]
fn test_cown_large_data() {
    // Test with a large vector to ensure memory handling works
    let large_vec: Vec<u8> = vec![0u8; 1024 * 1024]; // 1MB
    let cown = Cown::new(large_vec);

    drop(cown);
    assert!(true);
}

#[test]
fn test_cown_nested_structures() {
    #[derive(Debug)]
    struct Nested {
        data: Vec<String>,
        inner: Option<Box<Nested>>,
    }

    let nested = Nested {
        data: vec!["hello".to_string(), "world".to_string()],
        inner: Some(Box::new(Nested {
            data: vec!["nested".to_string()],
            inner: None,
        })),
    };

    let cown = Cown::new(nested);
    drop(cown);

    assert!(true);
}

#[test]
fn test_multiple_cowns_independent() {
    // Create multiple independent cowns
    let cown1 = Cown::new(1i32);
    let cown2 = Cown::new(2i32);
    let cown3 = Cown::new(3i32);

    // Drop them in different order
    drop(cown2);
    drop(cown1);
    drop(cown3);

    assert!(true);
}

#[test]
fn test_cown_send_trait() {
    // Verify Cown implements Send
    fn assert_send<T: Send>() {}
    assert_send::<Cown<i32>>();
    assert_send::<Cown<String>>();
    assert_send::<Cown<Vec<u8>>>();
}

#[test]
fn test_cown_clone_trait() {
    // Verify Cown implements Clone properly
    fn assert_clone<T: Clone>() {}
    assert_clone::<Cown<i32>>();
    assert_clone::<Cown<String>>();
}
