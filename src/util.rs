//! Inlined, dependency-free utilities: a fast FxHash-style hasher for the hot
//! BFS/search state maps, and a SplitMix64 PRNG for reproducible experiments.

use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, Hasher};

/// FxHash (the rustc hasher): fast, non-cryptographic, deterministic.
#[derive(Default)]
pub struct FxHasher {
    hash: u64,
}

const SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;

impl Hasher for FxHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.hash = (self.hash.rotate_left(5) ^ (b as u64)).wrapping_mul(SEED);
        }
    }
    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.hash = (self.hash.rotate_left(5) ^ i).wrapping_mul(SEED);
    }
    #[inline]
    fn finish(&self) -> u64 {
        self.hash
    }
}

pub type FxBuildHasher = BuildHasherDefault<FxHasher>;
pub type FxHashMap<K, V> = HashMap<K, V, FxBuildHasher>;
pub type FxHashSet<K> = HashSet<K, FxBuildHasher>;

pub fn fxmap<K, V>() -> FxHashMap<K, V> {
    FxHashMap::default()
}
pub fn fxset<K>() -> FxHashSet<K> {
    FxHashSet::default()
}

/// SplitMix64: tiny seedable PRNG (reproducible; not Python-compatible).
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng {
            state: seed.wrapping_add(0x9E37_79B9_7F4A_7C15),
        }
    }
    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    /// Uniform-ish in `[0, n)` (negligible modulo bias for our small `n`).
    #[inline]
    pub fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }
    #[inline]
    pub fn f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / ((1u64 << 53) as f64)
    }
    /// Fisher-Yates shuffle.
    pub fn shuffle<T>(&mut self, s: &mut [T]) {
        for i in (1..s.len()).rev() {
            let j = self.below(i + 1);
            s.swap(i, j);
        }
    }
    /// A random permutation `1..=n` as a deck.
    pub fn perm(&mut self, n: usize) -> Vec<u8> {
        let mut p: Vec<u8> = (1..=n as u8).collect();
        self.shuffle(&mut p);
        p
    }
}
