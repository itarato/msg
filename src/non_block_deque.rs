use std::{
    collections::VecDeque,
    sync::{Condvar, Mutex},
    time::Duration,
};

pub struct NonBlockDeque<T> {
    queue: Mutex<VecDeque<T>>,
    pull_cond: Condvar,
    pull_cond_mutex: Mutex<()>,
    timeout: Duration,
}

impl<T> NonBlockDeque<T> {
    pub fn new(timeout: Duration) -> NonBlockDeque<T> {
        NonBlockDeque {
            queue: Mutex::new(VecDeque::new()),
            pull_cond: Condvar::new(),
            pull_cond_mutex: Mutex::new(()),
            timeout,
        }
    }

    pub fn push(&self, e: T) {
        self.queue.lock().expect("lock queue for push").push_back(e);
        self.pull_cond.notify_one();
    }

    pub fn pop(&self) -> Option<T> {
        {
            if let Some(e) = self.queue.lock().expect("lock queue for pop").pop_front() {
                return Some(e);
            }
        }

        let _ = self
            .pull_cond
            .wait_timeout(
                self.pull_cond_mutex
                    .lock()
                    .expect("locking pull cond mutex"),
                self.timeout,
            )
            .expect("wait for condvar");

        self.queue.lock().expect("lock queue for pop").pop_front()
    }
}

#[cfg(test)]
mod test {
    use std::{sync::Arc, thread};

    use crate::non_block_deque::*;

    #[test]
    fn test_basics() {
        let queue: Arc<NonBlockDeque<i32>> =
            Arc::new(NonBlockDeque::new(Duration::from_millis(10)));

        println!("Start");

        thread::scope(|s| {
            s.spawn({
                let queue = queue.clone();
                move || {
                    let mut count = 0usize;

                    loop {
                        if let Some(i) = queue.pop() {
                            println!("Pop hit: {}", i);
                            count += 1;
                            if count >= 10 {
                                break;
                            }
                        } else {
                            println!("Pop miss.");
                        }
                    }

                    // Symbolical.
                    assert!(count == 10);
                }
            });

            s.spawn({
                let queue = queue.clone();
                move || {
                    for i in 0..10 {
                        queue.push(i);
                        println!("Push");
                        thread::sleep(Duration::from_millis(15));
                    }
                }
            });
        });
    }
}
