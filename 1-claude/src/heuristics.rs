//! Admissible heuristics: `h0 <= h_best <= h_joint`, all lower bounds on the
//! optimal move count (`h = h0 + 2*(lower bound on forced bounces)`). See
//! docs/NOTES.md §I.4. `h_joint` reduces the bounce bound to a constrained OCT
//! on the soft-conflict graph (`oct::exact_oct`).

use std::collections::HashSet;

use crate::machine::{base_len, Card, State};
use crate::oct::{exact_oct, Graph, PreColour, Vertex, DEFAULT_BUDGET};

/// Length of the longest strictly-increasing subsequence.
pub fn lis_len(seq: &[Card]) -> usize {
    let mut tails: Vec<Card> = Vec::new();
    for &x in seq {
        let i = tails.partition_point(|&t| t < x); // bisect_left (cards distinct)
        if i == tails.len() {
            tails.push(x);
        } else {
            tails[i] = x;
        }
    }
    tails.len()
}

/// Cards that are *buried*: a smaller card sits below them in the same buffer.
fn buried_set(a: &[Card], b: &[Card]) -> HashSet<Card> {
    let mut out = HashSet::new();
    for buf in [a, b] {
        for i in 0..buf.len() {
            let x = buf[i];
            if buf[..i].iter().any(|&y| y < x) {
                out.insert(x);
            }
        }
    }
    out
}

/// Base charge: each above-base deck card costs >= 2, each buffer card >= 1.
pub fn h0(s: &State) -> u32 {
    let k = base_len(&s.d);
    (2 * (s.d.len() - k) + s.a.len() + s.b.len()) as u32
}

/// Above-base deck values read top-to-bottom (the forced first-departure order).
fn pi_seq(s: &State) -> Vec<Card> {
    let k = base_len(&s.d);
    s.d[k..].iter().rev().copied().collect()
}

/// Longest mutual-conflict clique, as the longest chain of the conflict
/// comparability graph (kind 0 = above-base deck in pi order, 1 = A, 2 = B).
fn clique_chain(s: &State) -> usize {
    let pi = pi_seq(s);
    let mut nodes: Vec<(Card, u8, usize)> = Vec::new();
    for (pos, &v) in pi.iter().enumerate() {
        nodes.push((v, 0, pos));
    }
    for (d, &v) in s.a.iter().enumerate() {
        nodes.push((v, 1, d));
    }
    for (d, &v) in s.b.iter().enumerate() {
        nodes.push((v, 2, d));
    }
    let n = nodes.len();
    if n == 0 {
        return 0;
    }
    let edge = |i: usize, j: usize| -> bool {
        let (vi, ki, xi) = nodes[i];
        let (vj, kj, xj) = nodes[j];
        if ki == 0 && kj == 0 {
            if xi < xj {
                vi < vj
            } else {
                vj < vi
            }
        } else if ki == 0 && kj >= 1 {
            vj < vi
        } else if kj == 0 && ki >= 1 {
            vi < vj
        } else if ki >= 1 && kj >= 1 {
            if ki != kj {
                false
            } else if xi < xj {
                vi < vj
            } else {
                vj < vi
            }
        } else {
            false
        }
    };
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by_key(|&i| nodes[i].0);
    let mut best = vec![1usize; n];
    let mut ans = 0;
    for a in 0..n {
        let i = order[a];
        for &j in &order[..a] {
            if edge(i, j) && best[j] + 1 > best[i] {
                best[i] = best[j] + 1;
            }
        }
        ans = ans.max(best[i]);
    }
    ans
}

/// The "max" heuristic: `h0 + 2*max((LIS-2)+buried, clique-2)`.
pub fn h_best(s: &State) -> u32 {
    let pi = pi_seq(s);
    let buried = buried_set(&s.a, &s.b);
    let disjoint = lis_len(&pi).saturating_sub(2) + buried.len();
    let clique = clique_chain(s);
    h0(s) + 2 * disjoint.max(clique.saturating_sub(2)) as u32
}

/// Build the soft-conflict graph: non-buried buffer cards (pre-coloured by their
/// buffer) and all above-base deck cards (free). Soft edges: deck-deck increasing
/// in pi, and deck `d` vs non-buried buffer card `q < d`. Returns
/// `(values, adjacency, pre_colour, n_buried)`.
pub fn soft_conflict_graph(s: &State) -> (Vec<Vertex>, Graph, PreColour, usize) {
    let buried = buried_set(&s.a, &s.b);
    let pi = pi_seq(s);
    // nodes: (value, kind) with kind 0 = A, 1 = B, 2 = deck
    let mut nodes: Vec<(Card, u8)> = Vec::new();
    let mut pre: PreColour = PreColour::new();
    for &v in &s.a {
        if !buried.contains(&v) {
            nodes.push((v, 0));
            pre.insert(v as Vertex, 0);
        }
    }
    for &v in &s.b {
        if !buried.contains(&v) {
            nodes.push((v, 1));
            pre.insert(v as Vertex, 1);
        }
    }
    for &v in &pi {
        nodes.push((v, 2));
    }
    let vals: Vec<Vertex> = nodes.iter().map(|&(v, _)| v as Vertex).collect();
    let mut adj: Graph = Graph::new();
    for &v in &vals {
        adj.entry(v).or_default();
    }
    // recover deck pi-position by appearance order among kind-2 nodes
    let mut deck_pos: std::collections::HashMap<Card, usize> = std::collections::HashMap::new();
    let mut pos = 0;
    for &(v, kind) in &nodes {
        if kind == 2 {
            deck_pos.insert(v, pos);
            pos += 1;
        }
    }
    for i in 0..nodes.len() {
        for j in (i + 1)..nodes.len() {
            let (vi, ki) = nodes[i];
            let (vj, kj) = nodes[j];
            let e = if ki == 2 && kj == 2 {
                if deck_pos[&vi] < deck_pos[&vj] {
                    vi < vj
                } else {
                    vj < vi
                }
            } else if ki == 2 && kj < 2 {
                vj < vi
            } else if kj == 2 && ki < 2 {
                vi < vj
            } else {
                false
            };
            if e {
                adj.get_mut(&(vi as Vertex)).unwrap().insert(vj as Vertex);
                adj.get_mut(&(vj as Vertex)).unwrap().insert(vi as Vertex);
            }
        }
    }
    (vals, adj, pre, buried.len())
}

/// The joint heuristic: `h0 + 2*(buried + OCT_pre)`; dominates `h_best`.
pub fn h_joint(s: &State) -> u32 {
    let (vals, adj, pre, n_buried) = soft_conflict_graph(s);
    let (oct, _) = exact_oct(&vals, &adj, &pre, DEFAULT_BUDGET);
    h0(s) + 2 * (n_buried + oct) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_compound_example() {
        // compound state from the docs: opt = 17
        let s = State {
            d: vec![6, 2, 5],
            a: vec![4, 1],
            b: vec![3],
        };
        assert_eq!(h0(&s), 9);
        assert_eq!(h_best(&s), 11);
        assert_eq!(h_joint(&s), 13); // joint is strictly tighter here
    }

    #[test]
    fn reversal_is_exact_for_strong_heuristics() {
        for n in 3..=13 {
            let s = State::reversed_deck(n);
            assert_eq!(h_best(&s), 4 * (n as u32 - 1));
            assert_eq!(h_joint(&s), 4 * (n as u32 - 1));
            assert_eq!(h0(&s), 2 * n as u32);
        }
    }
}
