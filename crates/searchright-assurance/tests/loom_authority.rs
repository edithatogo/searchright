//! Concurrency permutation tests for single-writer authority transitions.

use loom::{
    model,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
};

#[test]
fn exactly_one_writer_can_commit_a_governed_transition() {
    model(|| {
        let state = Arc::new(AtomicUsize::new(0));
        let successful_writers = Arc::new(AtomicUsize::new(0));

        let first_state = Arc::clone(&state);
        let first_writers = Arc::clone(&successful_writers);
        let first = thread::spawn(move || {
            if first_state
                .compare_exchange(0, 1, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                first_writers.fetch_add(1, Ordering::AcqRel);
            }
        });

        let second_state = Arc::clone(&state);
        let second_writers = Arc::clone(&successful_writers);
        let second = thread::spawn(move || {
            if second_state
                .compare_exchange(0, 1, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                second_writers.fetch_add(1, Ordering::AcqRel);
            }
        });

        let first_result = first.join();
        let second_result = second.join();
        assert!(first_result.is_ok());
        assert!(second_result.is_ok());
        assert_eq!(state.load(Ordering::Acquire), 1);
        assert_eq!(successful_writers.load(Ordering::Acquire), 1);
    });
}
