//! Exact minimum odd-cycle transversal (OCT) with pre-coloured vertices, by
//! odd-cycle branch-and-bound. The soft-conflict graph is a comparability graph
//! (perfect), so it is bipartite iff triangle-free and its max clique equals its
//! longest value-chain; the chain bound `clique - 2 <= OCT` prunes the clique
//! case immediately, defeating the `3^k` blowup. Pre-colouring is encoded with
//! two undeletable terminal vertices `T0`, `T1`. See docs/NOTES.md §I.4.

use std::collections::{HashMap, HashSet, VecDeque};

/// Vertices are card values (`>= 1`); the terminals are negative sentinels.
pub type Vertex = i16;
/// An undirected graph as vertex → neighbour set.
pub type Graph = HashMap<Vertex, HashSet<Vertex>>;
/// A pre-colouring: vertex → fixed side (`0` or `1`).
pub type PreColour = HashMap<Vertex, u8>;

/// Undeletable terminal for colour side 0.
pub const T0: Vertex = -1;
/// Undeletable terminal for colour side 1.
pub const T1: Vertex = -2;

fn link(g: &mut Graph, a: Vertex, b: Vertex) {
    g.entry(a).or_default().insert(b);
    g.entry(b).or_default().insert(a);
}

/// Max clique = longest value-chain of the comparability graph induced on
/// `remaining`. `O(|remaining|^2)`.
fn clique(remaining: &HashSet<Vertex>, adj: &Graph) -> usize {
    if remaining.is_empty() {
        return 0;
    }
    let mut order: Vec<Vertex> = remaining.iter().copied().collect();
    order.sort_unstable();
    let mut best: HashMap<Vertex, usize> = HashMap::new();
    let mut ans = 0;
    for v in order {
        let mut b = 1;
        if let Some(nb) = adj.get(&v) {
            for &w in nb {
                if w < v && remaining.contains(&w) {
                    if let Some(&bw) = best.get(&w) {
                        if bw + 1 > b {
                            b = bw + 1;
                        }
                    }
                }
            }
        }
        best.insert(v, b);
        if b > ans {
            ans = b;
        }
    }
    ans
}

/// 2-colour `adj`. Return `None` if bipartite, else a vertex list forming an odd
/// closed walk (its vertex set meets every bipartization).
fn bipartite_or_odd_cycle(adj: &Graph) -> Option<Vec<Vertex>> {
    let mut colour: HashMap<Vertex, u8> = HashMap::new();
    let mut parent: HashMap<Vertex, Option<Vertex>> = HashMap::new();
    let mut srcs: Vec<Vertex> = adj.keys().copied().collect();
    srcs.sort_unstable(); // deterministic
    for src in srcs {
        if colour.contains_key(&src) {
            continue;
        }
        colour.insert(src, 0);
        parent.insert(src, None);
        let mut dq: VecDeque<Vertex> = VecDeque::new();
        dq.push_back(src);
        while let Some(u) = dq.pop_front() {
            let cu = colour[&u];
            let mut nb: Vec<Vertex> = adj[&u].iter().copied().collect();
            nb.sort_unstable();
            for w in nb {
                if let Some(&cw) = colour.get(&w) {
                    if cw == cu {
                        // odd cycle: climb to common ancestor
                        let mut au: Vec<Vertex> = Vec::new();
                        let mut seen: HashSet<Vertex> = HashSet::new();
                        let mut x = Some(u);
                        while let Some(xx) = x {
                            au.push(xx);
                            seen.insert(xx);
                            x = parent[&xx];
                        }
                        let mut cyc: Vec<Vertex> = Vec::new();
                        let mut x = Some(w);
                        while let Some(xx) = x {
                            if seen.contains(&xx) {
                                break;
                            }
                            cyc.push(xx);
                            x = parent[&xx];
                        }
                        let lca = x; // common ancestor (Some) or None
                        let mut path_u: Vec<Vertex> = Vec::new();
                        for a in au {
                            path_u.push(a);
                            if Some(a) == lca {
                                break;
                            }
                        }
                        path_u.extend(cyc);
                        return Some(path_u);
                    }
                } else {
                    colour.insert(w, 1 - cu);
                    parent.insert(w, Some(u));
                    dq.push_back(w);
                }
            }
        }
    }
    None
}

#[allow(clippy::too_many_arguments)]
fn bb(
    cur: &Graph,
    deleted: usize,
    remaining: &HashSet<Vertex>,
    adj: &Graph,
    budget: u64,
    best: &mut usize,
    calls: &mut u64,
    aborted: &mut bool,
) {
    if *aborted || deleted >= *best {
        return;
    }
    *calls += 1;
    if *calls > budget {
        *aborted = true;
        return;
    }
    let lb = deleted + clique(remaining, adj).saturating_sub(2);
    if lb >= *best {
        return;
    }
    match bipartite_or_odd_cycle(cur) {
        None => {
            *best = deleted;
        }
        Some(oc) => {
            let mut seen: HashSet<Vertex> = HashSet::new();
            for v in oc {
                if v == T0 || v == T1 || !seen.insert(v) {
                    continue;
                }
                let mut sub: Graph = HashMap::with_capacity(cur.len());
                for (&x, nb) in cur {
                    if x == v {
                        continue;
                    }
                    let mut s2 = nb.clone();
                    s2.remove(&v);
                    sub.insert(x, s2);
                }
                let mut rem2 = remaining.clone();
                rem2.remove(&v);
                bb(&sub, deleted + 1, &rem2, adj, budget, best, calls, aborted);
                if *aborted {
                    return;
                }
            }
        }
    }
}

/// Minimum vertices to delete so the graph 2-colours respecting `pre`. Exact
/// whenever the branch-and-bound finishes within `budget` node expansions;
/// otherwise returns the chain lower bound `clique - 2 <= OCT` (still admissible)
/// with `exact = false`. Returns `(value, exact)`.
pub fn exact_oct(vals: &[Vertex], adj: &Graph, pre: &PreColour, budget: u64) -> (usize, bool) {
    if vals.is_empty() {
        return (0, true);
    }
    let mut aug: Graph = HashMap::with_capacity(vals.len() + 2);
    for &v in vals {
        aug.insert(v, adj.get(&v).cloned().unwrap_or_default());
    }
    aug.insert(T0, HashSet::new());
    aug.insert(T1, HashSet::new());
    link(&mut aug, T0, T1);
    for &v in vals {
        match pre.get(&v) {
            Some(0) => link(&mut aug, v, T1),
            Some(1) => link(&mut aug, v, T0),
            _ => {}
        }
    }
    let mut best = vals.len();
    let mut calls = 0u64;
    let mut aborted = false;
    let remaining: HashSet<Vertex> = vals.iter().copied().collect();
    bb(
        &aug,
        0,
        &remaining,
        adj,
        budget,
        &mut best,
        &mut calls,
        &mut aborted,
    );
    if aborted {
        (clique(&remaining, adj).saturating_sub(2), false)
    } else {
        (best, true)
    }
}

/// Default branch-and-bound node-expansion budget before falling back.
pub const DEFAULT_BUDGET: u64 = 40_000;

// ----- reference oracle (exponential; tests only) -----------------------------

fn two_colourable(verts: &[Vertex], adj: &Graph, pre: &PreColour) -> bool {
    let vset: HashSet<Vertex> = verts.iter().copied().collect();
    let mut colour: HashMap<Vertex, u8> = HashMap::new();
    for (&v, &c) in pre {
        if vset.contains(&v) {
            colour.insert(v, c);
        }
    }
    let mut dq: VecDeque<Vertex> = colour.keys().copied().collect();
    // forced colours first, then free components
    let mut roots: Vec<Vertex> = verts.to_vec();
    let mut idx = 0;
    loop {
        while let Some(u) = dq.pop_front() {
            let cu = colour[&u];
            if let Some(nb) = adj.get(&u) {
                for &w in nb {
                    match colour.get(&w) {
                        Some(&cw) => {
                            if cw == cu {
                                return false;
                            }
                        }
                        None => {
                            colour.insert(w, 1 - cu);
                            dq.push_back(w);
                        }
                    }
                }
            }
        }
        // seed next uncoloured vertex
        while idx < roots.len() && colour.contains_key(&roots[idx]) {
            idx += 1;
        }
        if idx >= roots.len() {
            break;
        }
        let v = roots[idx];
        colour.insert(v, 0);
        dq.push_back(v);
    }
    let _ = &mut roots;
    true
}

fn combinations(vals: &[Vertex], d: usize) -> Vec<Vec<Vertex>> {
    let mut out = Vec::new();
    let mut cur = Vec::new();
    fn rec(
        vals: &[Vertex],
        d: usize,
        start: usize,
        cur: &mut Vec<Vertex>,
        out: &mut Vec<Vec<Vertex>>,
    ) {
        if cur.len() == d {
            out.push(cur.clone());
            return;
        }
        for i in start..vals.len() {
            cur.push(vals[i]);
            rec(vals, d, i + 1, cur, out);
            cur.pop();
        }
    }
    rec(vals, d, 0, &mut cur, &mut out);
    out
}

/// Reference OCT by exhaustive deletion search (the test oracle).
pub fn oct_pre_bruteforce(vals: &[Vertex], adj: &Graph, pre: &PreColour) -> usize {
    for d in 0..=vals.len() {
        for drop in combinations(vals, d) {
            let dropped: HashSet<Vertex> = drop.iter().copied().collect();
            let kept: Vec<Vertex> = vals
                .iter()
                .copied()
                .filter(|v| !dropped.contains(v))
                .collect();
            let mut sub: Graph = HashMap::new();
            for &v in &kept {
                let nb: HashSet<Vertex> = adj
                    .get(&v)
                    .map(|s| s.iter().copied().filter(|w| !dropped.contains(w)).collect())
                    .unwrap_or_default();
                sub.insert(v, nb);
            }
            let pre_kept: PreColour = kept
                .iter()
                .filter_map(|v| pre.get(v).map(|&c| (*v, c)))
                .collect();
            if two_colourable(&kept, &sub, &pre_kept) {
                return d;
            }
        }
    }
    vals.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn complete(m: i16) -> (Vec<Vertex>, Graph) {
        let vals: Vec<Vertex> = (1..=m).collect();
        let mut adj: Graph = HashMap::new();
        for &v in &vals {
            adj.insert(v, vals.iter().copied().filter(|&w| w != v).collect());
        }
        (vals, adj)
    }

    #[test]
    fn cliques_need_m_minus_2() {
        for m in [3, 5, 10, 20, 52] {
            let (vals, adj) = complete(m);
            let (val, exact) = exact_oct(&vals, &adj, &PreColour::new(), DEFAULT_BUDGET);
            assert_eq!(val, (m - 2) as usize);
            assert!(exact);
        }
    }

    #[test]
    fn edge_cases() {
        assert_eq!(
            exact_oct(&[], &Graph::new(), &PreColour::new(), DEFAULT_BUDGET),
            (0, true)
        );
        let mut adj: Graph = HashMap::new();
        for v in [1, 2, 3] {
            adj.insert(v, HashSet::new());
        }
        assert_eq!(
            exact_oct(&[1, 2, 3], &adj, &PreColour::new(), DEFAULT_BUDGET).0,
            0
        );
    }
}
