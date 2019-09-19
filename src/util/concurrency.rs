use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};

/// Provides synchronization on a one-shot condition.
///
/// Until the condition is met, threads waiting on it are blocked.
///
/// Once the condition is triggered, all waiting threads are awaked and any
/// other waits on the condition will not block.
#[derive(Clone)]
pub struct Condition {
	condition: Arc<(Mutex<bool>, Condvar)>,
	completed: Arc<AtomicBool>,
}

impl Default for Condition {
	fn default() -> Condition {
		Condition {
			condition: Arc::new((Mutex::new(false), Condvar::new())),
			completed: Default::default(),
		}
	}
}

impl Condition {
	/// Wait on the condition.
	pub fn wait(&self) {
		// Do a quick check at the start to prevent mutex acquire and cloning
		// when the condition has already been met.
		if self.completed.load(Ordering::SeqCst) {
			return;
		}

		// Check the condition state in a loop, blocking until it is completed.
		let condition = self.condition.clone();
		let &(ref mutex, ref condvar) = &*condition;
		let mut condition = mutex.lock().unwrap();
		while !*condition {
			condition = condvar.wait(condition).unwrap();
		}
	}

	/// Triggers the condition, waking all threads that are waiting on it.
	///
	/// After this, calls to [wait] will no longer block.
	pub fn trigger(&self) {
		self.completed.store(true, Ordering::SeqCst);

		let condition = self.condition.clone();
		let &(ref mutex, ref condvar) = &*condition;
		let mut condition = mutex.lock().unwrap();
		*condition = true;
		condvar.notify_all();
	}

	/// Reset the condition.
	///
	/// After this, calls to [wait] will block until [trigger] is called.
	pub fn reset(&self) {
		let condition = self.condition.clone();
		let &(ref mutex, ref _condvar) = &*condition;
		let mut condition = mutex.lock().unwrap();
		*condition = false;

		self.completed.store(false, Ordering::SeqCst);
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	use std::sync::atomic::{AtomicU64, Ordering};
	use std::sync::{mpsc::channel, Arc};
	use std::thread::spawn;

	#[test]
	fn test_condition() {
		let cond = Condition::default();
		let value = Arc::new(AtomicU64::default());

		let (tx, rx) = channel();
		for _ in 0..5 {
			let cond = cond.clone();
			let tx = tx.clone();
			let value = value.clone();
			spawn(move || {
				cond.wait();
				let value = value.load(Ordering::SeqCst);
				tx.send(value).unwrap();
			});
		}
		drop(tx);

		value.store(42, Ordering::SeqCst);
		cond.trigger();
		cond.wait(); // Should not block after trigger

		let mut count = 0;
		for it in rx {
			count += 1;
			assert_eq!(it, 42);
		}
		assert_eq!(count, 5);

		// Test reset
		cond.reset();

		let (tx, rx) = channel();
		for _ in 0..20 {
			let cond = cond.clone();
			let tx = tx.clone();
			let value = value.clone();
			spawn(move || {
				cond.wait();
				let value = value.load(Ordering::SeqCst);
				tx.send(value).unwrap();
			});
		}
		drop(tx);

		value.store(100, Ordering::SeqCst);
		cond.trigger();
		cond.wait(); // Should not block after trigger

		let mut count = 0;
		for it in rx {
			count += 1;
			assert_eq!(it, 100);
		}
		assert_eq!(count, 20);
	}
}
