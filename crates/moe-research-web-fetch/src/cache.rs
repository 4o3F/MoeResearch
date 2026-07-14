use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use crate::WebFetchDocument;

struct CacheEntry {
    document: Arc<WebFetchDocument>,
    inserted_at: Instant,
    sequence: u64,
}

pub(crate) struct DocumentCache {
    ttl: Duration,
    max_entries: usize,
    state: RwLock<CacheState>,
}

#[derive(Default)]
struct CacheState {
    entries: HashMap<String, CacheEntry>,
    next_sequence: u64,
}

impl DocumentCache {
    pub(crate) fn new(ttl: Duration, max_entries: usize) -> Self {
        Self {
            ttl,
            max_entries,
            state: RwLock::new(CacheState::default()),
        }
    }

    pub(crate) fn is_enabled(&self) -> bool {
        !self.ttl.is_zero() && self.max_entries > 0
    }

    pub(crate) fn get(&self, key: &str) -> Option<Arc<WebFetchDocument>> {
        if self.ttl.is_zero() {
            return None;
        }
        {
            let state = self
                .state
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let entry = state.entries.get(key)?;
            if entry.inserted_at.elapsed() < self.ttl {
                return Some(entry.document.clone());
            }
        }

        let mut state = self
            .state
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match state.entries.get(key) {
            Some(entry) if entry.inserted_at.elapsed() < self.ttl => Some(entry.document.clone()),
            Some(_) => {
                state.entries.remove(key);
                None
            }
            None => None,
        }
    }

    pub(crate) fn insert(&self, key: String, document: Arc<WebFetchDocument>) {
        if self.ttl.is_zero() || self.max_entries == 0 {
            return;
        }
        let mut state = self
            .state
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state
            .entries
            .retain(|_, entry| entry.inserted_at.elapsed() < self.ttl);
        state.next_sequence = state.next_sequence.saturating_add(1);
        let sequence = state.next_sequence;
        state.entries.insert(
            key,
            CacheEntry {
                document,
                inserted_at: Instant::now(),
                sequence,
            },
        );
        while state.entries.len() > self.max_entries {
            let oldest = state
                .entries
                .iter()
                .min_by_key(|(_, entry)| entry.sequence)
                .map(|(key, _)| key.clone());
            let Some(oldest) = oldest else { break };
            state.entries.remove(&oldest);
        }
    }
}
