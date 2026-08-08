
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use super::types::{GeniusArtist, GeniusHit, GeniusPersonHit, GeniusSong};

const SONG_TTL: Duration = Duration::from_secs(60 * 60 * 6);
const ARTIST_TTL: Duration = Duration::from_secs(60 * 60 * 12);
const MATCH_TTL: Duration = Duration::from_secs(60 * 60 * 24 * 7);
const SEARCH_TTL: Duration = Duration::from_secs(60 * 10);

const SONG_CAP: usize = 240;
const ARTIST_CAP: usize = 120;
const MATCH_CAP: usize = 2000;
const SEARCH_CAP: usize = 120;

struct Entry<V> {
    value: V,
    born: Instant,
}

pub struct Store<K, V> {
    items: Mutex<HashMap<K, Entry<V>>>,
    ttl: Duration,
    cap: usize,
}

impl<K: Eq + Hash + Clone, V: Clone> Store<K, V> {
    fn new(ttl: Duration, cap: usize) -> Self {
        Self {
            items: Mutex::new(HashMap::new()),
            ttl,
            cap,
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let mut items = self.items.lock().ok()?;
        let fresh = items
            .get(key)
            .filter(|entry| entry.born.elapsed() <= self.ttl)
            .map(|entry| entry.value.clone());
        if fresh.is_none() {
            items.remove(key);
        }
        fresh
    }

    pub fn put(&self, key: K, value: V) {
        let Ok(mut items) = self.items.lock() else {
            return;
        };

        if items.len() >= self.cap {
            let ttl = self.ttl;
            items.retain(|_, entry| entry.born.elapsed() <= ttl);
        }
        while items.len() >= self.cap {
            let oldest = items
                .iter()
                .min_by_key(|(_, entry)| entry.born)
                .map(|(key, _)| key.clone());
            match oldest {
                Some(key) => {
                    items.remove(&key);
                }
                None => break,
            }
        }

        items.insert(
            key,
            Entry {
                value,
                born: Instant::now(),
            },
        );
    }

    pub fn clear(&self) {
        if let Ok(mut items) = self.items.lock() {
            items.clear();
        }
    }
}

pub fn songs() -> &'static Store<u64, GeniusSong> {
    static CACHE: OnceLock<Store<u64, GeniusSong>> = OnceLock::new();
    CACHE.get_or_init(|| Store::new(SONG_TTL, SONG_CAP))
}

pub fn artists() -> &'static Store<u64, GeniusArtist> {
    static CACHE: OnceLock<Store<u64, GeniusArtist>> = OnceLock::new();
    CACHE.get_or_init(|| Store::new(ARTIST_TTL, ARTIST_CAP))
}

pub fn matches() -> &'static Store<String, u64> {
    static CACHE: OnceLock<Store<String, u64>> = OnceLock::new();
    CACHE.get_or_init(|| Store::new(MATCH_TTL, MATCH_CAP))
}

pub fn song_search() -> &'static Store<String, Vec<GeniusHit>> {
    static CACHE: OnceLock<Store<String, Vec<GeniusHit>>> = OnceLock::new();
    CACHE.get_or_init(|| Store::new(SEARCH_TTL, SEARCH_CAP))
}

pub fn people_search() -> &'static Store<String, Vec<GeniusPersonHit>> {
    static CACHE: OnceLock<Store<String, Vec<GeniusPersonHit>>> = OnceLock::new();
    CACHE.get_or_init(|| Store::new(SEARCH_TTL, SEARCH_CAP))
}

pub fn clear_all() {
    songs().clear();
    artists().clear();
    matches().clear();
    song_search().clear();
    people_search().clear();
}
