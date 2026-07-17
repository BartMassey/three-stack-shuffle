//! The whole-cycle (permutation-distance) model. One cycle = two reversed
//! subsequences interleaved; `e` is reachable from `d` in one cycle iff
//! `LIS(d^{-1} e) <= 2`. The reachability graph is the undirected Cayley graph of
//! `S_n` with the 123-avoiding generators, and `f(pi)` is the word length there.
//! Permutations are 0-indexed. See docs/NOTES.md Part II.

use std::collections::VecDeque;

use crate::util::{fxmap, fxset, FxHashMap};

/// A permutation in one-line notation (0-indexed values).
pub type Perm = Vec<u8>;

/// Length of the longest strictly-increasing subsequence.
pub fn lis(p: &[u8]) -> usize {
    let mut tails: Vec<u8> = Vec::new();
    for &x in p {
        let i = tails.partition_point(|&t| t < x);
        if i == tails.len() {
            tails.push(x);
        } else {
            tails[i] = x;
        }
    }
    tails.len()
}

/// The relative permutation `d^{-1} e`.
pub fn rel(d: &[u8], e: &[u8]) -> Perm {
    let mut pos = vec![0u8; d.len()];
    for (i, &c) in d.iter().enumerate() {
        pos[c as usize] = i as u8;
    }
    e.iter().map(|&x| pos[x as usize]).collect()
}

/// Is `e` reachable from `d` in exactly one cycle?
pub fn one_cycle_ok(d: &[u8], e: &[u8]) -> bool {
    lis(&rel(d, e)) <= 2
}

fn all_perms(n: usize) -> Vec<Perm> {
    let mut out = Vec::new();
    let mut p: Perm = (0..n as u8).collect();
    fn rec(p: &mut Perm, i: usize, out: &mut Vec<Perm>) {
        if i == p.len() {
            out.push(p.clone());
            return;
        }
        for j in i..p.len() {
            p.swap(i, j);
            rec(p, i + 1, out);
            p.swap(i, j);
        }
    }
    rec(&mut p, 0, &mut out);
    out
}

/// The generating set `C = {sigma : LIS(sigma) <= 2}` (123-avoiding; `|C| = C_n`).
pub fn generators(n: usize) -> Vec<Perm> {
    all_perms(n).into_iter().filter(|p| lis(p) <= 2).collect()
}

#[inline]
fn compose(g: &[u8], c: &[u8]) -> Perm {
    c.iter().map(|&ci| g[ci as usize]).collect()
}

/// `{permutation: f}` for all of `S_n` by BFS in the Cayley graph.
pub fn cycle_distances(n: usize) -> FxHashMap<Perm, u32> {
    let gens = generators(n);
    let idt: Perm = (0..n as u8).collect();
    let mut dist: FxHashMap<Perm, u32> = fxmap();
    dist.insert(idt.clone(), 0);
    let mut q: VecDeque<Perm> = VecDeque::new();
    q.push_back(idt);
    while let Some(g) = q.pop_front() {
        let d = dist[&g];
        for c in &gens {
            let h = compose(&g, c);
            if !dist.contains_key(&h) {
                dist.insert(h.clone(), d + 1);
                q.push_back(h);
            }
        }
    }
    dist
}

/// `D(n) = max_pi f(pi)`.
pub fn cycle_diameter(n: usize) -> u32 {
    *cycle_distances(n).values().max().unwrap()
}

/// Minimum cycles to reach `pi` from the identity (word length over `C`).
pub fn f(pi: &[u8]) -> u32 {
    cycle_distances(pi.len())[pi]
}

fn combinations(n: usize, k: usize) -> Vec<Vec<usize>> {
    let mut out = Vec::new();
    let mut cur = Vec::new();
    fn rec(n: usize, k: usize, start: usize, cur: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
        if cur.len() == k {
            out.push(cur.clone());
            return;
        }
        for i in start..n {
            cur.push(i);
            rec(n, k, i + 1, cur, out);
            cur.pop();
        }
    }
    rec(n, k, 0, &mut cur, &mut out);
    out
}

/// The one-cycle reachable set of `d`, by direct construction (the cross-check
/// oracle for `one_cycle_ok`; exponential).
pub fn one_cycle_neighbors_bruteforce(d: &[u8]) -> crate::util::FxHashSet<Perm> {
    let n = d.len();
    let mut reach = fxset();
    for bits in 0u32..(1u32 << n) {
        let a: Vec<u8> = (0..n)
            .filter(|&i| (bits >> i) & 1 == 1)
            .map(|i| d[i])
            .collect();
        let b: Vec<u8> = (0..n)
            .filter(|&i| (bits >> i) & 1 == 0)
            .map(|i| d[i])
            .collect();
        let ra: Vec<u8> = a.iter().rev().copied().collect();
        let rb: Vec<u8> = b.iter().rev().copied().collect();
        for pos in combinations(n, ra.len()) {
            let mut out = vec![0u8; n];
            let pset: crate::util::FxHashSet<usize> = pos.iter().copied().collect();
            let (mut ia, mut ib) = (0, 0);
            for (k, slot) in out.iter_mut().enumerate() {
                if pset.contains(&k) {
                    *slot = ra[ia];
                    ia += 1;
                } else {
                    *slot = rb[ib];
                    ib += 1;
                }
            }
            reach.insert(out);
        }
    }
    reach
}

#[cfg(test)]
mod tests {
    use super::*;

    const CATALAN: [usize; 9] = [1, 1, 2, 5, 14, 42, 132, 429, 1430];

    #[test]
    fn diameter_sequence() {
        let d: Vec<u32> = (2..=8).map(cycle_diameter).collect();
        assert_eq!(d, vec![1, 1, 2, 2, 2, 3, 3]);
    }

    #[test]
    #[allow(clippy::needless_range_loop)] // n is both the index and the argument
    fn generators_are_catalan() {
        for n in 2..=8 {
            assert_eq!(generators(n).len(), CATALAN[n]);
        }
    }

    #[test]
    #[allow(clippy::needless_range_loop)] // n is both the index and the argument
    fn bruteforce_equals_lis_test() {
        for n in 2..=5 {
            let perms = all_perms(n);
            for d in &perms {
                let bf = one_cycle_neighbors_bruteforce(d);
                let alg: crate::util::FxHashSet<Perm> = perms
                    .iter()
                    .filter(|e| one_cycle_ok(d, e))
                    .cloned()
                    .collect();
                assert_eq!(bf, alg);
            }
        }
        // identity neighbours == Catalan
        for n in 2..=6 {
            let idt: Perm = (0..n as u8).collect();
            assert_eq!(one_cycle_neighbors_bruteforce(&idt).len(), CATALAN[n]);
        }
    }

    #[test]
    fn lis_breakdown_n7() {
        // distance-3 permutations at n=7 all have LIS 3 (the "225 states")
        let dist = cycle_distances(7);
        let far: Vec<&Perm> = dist
            .iter()
            .filter(|(_, &d)| d == 3)
            .map(|(p, _)| p)
            .collect();
        assert_eq!(far.len(), 225);
        assert!(far.iter().all(|p| lis(p) == 3));
    }
}
