use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use tokio::sync::{Mutex as AsyncMutex, OwnedMutexGuard};

#[derive(Default)]
pub(crate) struct FetchCoordinator {
    locks: Arc<Mutex<HashMap<String, Arc<AsyncMutex<()>>>>>,
}

impl FetchCoordinator {
    pub(crate) async fn acquire(&self, key: &str) -> FetchPermit {
        let lock = {
            let mut locks = lock_map(&self.locks);
            locks
                .entry(key.to_owned())
                .or_insert_with(|| Arc::new(AsyncMutex::new(())))
                .clone()
        };
        let guard = lock.clone().lock_owned().await;
        FetchPermit {
            key: key.to_owned(),
            lock,
            locks: self.locks.clone(),
            guard: Some(guard),
        }
    }
}

pub(crate) struct FetchPermit {
    key: String,
    lock: Arc<AsyncMutex<()>>,
    locks: Arc<Mutex<HashMap<String, Arc<AsyncMutex<()>>>>>,
    guard: Option<OwnedMutexGuard<()>>,
}

impl Drop for FetchPermit {
    fn drop(&mut self) {
        self.guard.take();
        let mut locks = lock_map(&self.locks);
        let is_current = locks
            .get(&self.key)
            .is_some_and(|lock| Arc::ptr_eq(lock, &self.lock));
        if is_current && Arc::strong_count(&self.lock) == 2 {
            locks.remove(&self.key);
        }
    }
}

fn lock_map<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
