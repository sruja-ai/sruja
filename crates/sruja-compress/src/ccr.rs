//! Reversible compression store (CCR). The original text is content-addressed
//! (sha256); the compressed message carries a [`CcrHandle`] the model can
//! present to a `retrieve` tool to fetch the original on demand.

use std::collections::HashMap;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::CompressError;

/// Opaque handle injected into the compressed message. The model cites this in
/// a `headroom_retrieve`-style tool call.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CcrHandle(pub String);

impl CcrHandle {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A reversible store. The in-memory impl is the default; a persistent impl
/// (e.g. sled/sqlite) lives at the agent layer where the runtime already has a
/// store.
pub trait CcrStore: Send + Sync {
    fn put(&self, original: &str) -> Result<CcrHandle, CompressError>;
    fn get(&self, handle: &CcrHandle) -> Result<Option<String>, CompressError>;
}

/// In-memory store. Fine for a single agent session. Not persistent.
#[derive(Default)]
pub struct InMemoryCcrStore {
    map: Mutex<HashMap<String, String>>,
}

impl CcrStore for InMemoryCcrStore {
    fn put(&self, original: &str) -> Result<CcrHandle, CompressError> {
        let digest = sha256_hex(original);
        self.map
            .lock()
            .map_err(|e| CompressError::Ccr(format!("lock poisoned: {e}")))?
            .insert(digest.clone(), original.to_string());
        Ok(CcrHandle(digest))
    }

    fn get(&self, handle: &CcrHandle) -> Result<Option<String>, CompressError> {
        Ok(self
            .map
            .lock()
            .map_err(|e| CompressError::Ccr(format!("lock poisoned: {e}")))?
            .get(&handle.0)
            .cloned())
    }
}

/// Bounded LRU-backed CCR store. Evicts oldest entries when the limit is hit.
/// Configurable capacity prevents unbounded memory growth in long sessions.
pub struct BoundedCcrStore {
    entries: Mutex<lru::LruCache<String, String>>,
}

impl BoundedCcrStore {
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: Mutex::new(lru::LruCache::new(
                std::num::NonZeroUsize::new(capacity.max(1)).unwrap(),
            )),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self::new(capacity)
    }
}

impl Default for BoundedCcrStore {
    fn default() -> Self {
        Self::with_capacity(1000)
    }
}

impl CcrStore for BoundedCcrStore {
    fn put(&self, original: &str) -> Result<CcrHandle, CompressError> {
        let digest = sha256_hex(original);
        self.entries
            .lock()
            .map_err(|e| CompressError::Ccr(format!("lock poisoned: {e}")))?
            .put(digest.clone(), original.to_string());
        Ok(CcrHandle(digest))
    }

    fn get(&self, handle: &CcrHandle) -> Result<Option<String>, CompressError> {
        Ok(self
            .entries
            .lock()
            .map_err(|e| CompressError::Ccr(format!("lock poisoned: {e}")))?
            .get(&handle.0)
            .cloned())
    }
}

fn sha256_hex(s: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(s);
    let bytes = h.finalize();
    let mut out = String::with_capacity(bytes.len() * 2);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_memory_round_trip() {
        let store = InMemoryCcrStore::default();
        let handle = store.put("original content").unwrap();
        assert_eq!(store.get(&handle).unwrap(), Some("original content".to_string()));
    }

    #[test]
    fn in_memory_content_addressed() {
        let store = InMemoryCcrStore::default();
        let h1 = store.put("same").unwrap();
        let h2 = store.put("same").unwrap();
        assert_eq!(h1, h2, "same content must produce same handle");
    }

    #[test]
    fn bounded_evicts_oldest() {
        let store = BoundedCcrStore::new(2);
        let h1 = store.put("first").unwrap();
        let _h2 = store.put("second").unwrap();
        let _h3 = store.put("third").unwrap();

        assert_eq!(
            store.get(&h1).unwrap(),
            None,
            "h1 should have been evicted (capacity=2)"
        );
    }

    #[test]
    fn bounded_lru_access_keeps_entry() {
        let store = BoundedCcrStore::new(2);
        let h1 = store.put("first").unwrap();
        let _h2 = store.put("second").unwrap();

        store.get(&h1).unwrap();

        let _h3 = store.put("third").unwrap();

        assert_eq!(
            store.get(&h1).unwrap(),
            Some("first".to_string()),
            "h1 was accessed recently, should survive eviction"
        );
    }

    #[test]
    fn bounded_round_trip() {
        let store = BoundedCcrStore::default();
        let handle = store.put("bounded original").unwrap();
        assert_eq!(
            store.get(&handle).unwrap(),
            Some("bounded original".to_string())
        );
    }
}
