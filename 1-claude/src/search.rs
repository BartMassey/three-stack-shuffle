//! Exact search: backward BFS (small `n`) and IDA* driven by an admissible
//! heuristic. IDA* uses parent-move pruning, on-path cycle detection, and
//! pathmax (the strong heuristics are admissible but not consistent).

use std::collections::VecDeque;

use crate::machine::{base_len, Move, State};
use crate::util::{fxmap, fxset, FxHashMap, FxHashSet};

/// Exact distance-to-goal for every reachable state with `n` cards, by BFS
/// backward from the sorted goal (the move graph is undirected). Feasible only
/// for small `n`; used for verification.
pub fn bfs_dist(n: usize) -> FxHashMap<State, u32> {
    let goal = State::goal(n);
    let mut dist: FxHashMap<State, u32> = fxmap();
    dist.insert(goal.clone(), 0);
    let mut q: VecDeque<State> = VecDeque::new();
    q.push_back(goal);
    while let Some(s) = q.pop_front() {
        let d = dist[&s];
        for (t, _) in s.successors() {
            if !dist.contains_key(&t) {
                dist.insert(t.clone(), d + 1);
                q.push_back(t);
            }
        }
    }
    dist
}

enum R {
    Found,
    GaveUp,
    Bound(u32),
}

#[allow(clippy::too_many_arguments)]
fn dfs<H: Fn(&State) -> u32>(
    s: &State,
    g: u32,
    bound: u32,
    last: Option<Move>,
    h_s: u32,
    goal: &State,
    h: &H,
    node_cap: u64,
    nodes: &mut u64,
    on_path: &mut FxHashSet<State>,
) -> R {
    let f = g + h_s;
    if f > bound {
        return R::Bound(f);
    }
    if s == goal {
        return R::Found;
    }
    *nodes += 1;
    if *nodes > node_cap {
        return R::GaveUp;
    }
    let mut best = u32::MAX;
    for (t, mv) in s.successors() {
        if let Some(lm) = last {
            if mv == lm.inv() {
                continue; // don't undo the previous move
            }
        }
        if on_path.contains(&t) {
            continue; // no cycles on the current path
        }
        let h_t = h(&t);
        let ph = h_t.max(h_s.saturating_sub(1)); // pathmax
        if g + 1 + ph > bound {
            best = best.min(g + 1 + ph);
            continue;
        }
        on_path.insert(t.clone());
        let r = dfs(
            &t,
            g + 1,
            bound,
            Some(mv),
            ph,
            goal,
            h,
            node_cap,
            nodes,
            on_path,
        );
        on_path.remove(&t);
        match r {
            R::Found => return R::Found,
            R::GaveUp => return R::GaveUp,
            R::Bound(rb) => best = best.min(rb),
        }
    }
    R::Bound(best)
}

/// IDA* from `start` to the sorted goal using `h` (must be admissible for the
/// returned cost to be optimal). Returns `(cost, nodes_expanded)`; `cost` is
/// `None` if the node cap was hit before a solution was found.
pub fn ida_star<H: Fn(&State) -> u32>(start: &State, h: &H, node_cap: u64) -> (Option<u32>, u64) {
    let goal = State::goal(start.size());
    let mut nodes = 0u64;
    let mut on_path: FxHashSet<State> = fxset();
    on_path.insert(start.clone());
    let mut bound = h(start);
    loop {
        match dfs(
            start,
            0,
            bound,
            None,
            bound,
            &goal,
            h,
            node_cap,
            &mut nodes,
            &mut on_path,
        ) {
            R::Found => return (Some(bound), nodes),
            R::GaveUp => return (None, nodes),
            R::Bound(b) => {
                if b == u32::MAX {
                    return (None, nodes);
                }
                bound = b;
            }
        }
    }
}

/// Helper: is `state` a clean start deck (buffers empty)? Used by experiments.
pub fn is_start_deck(state: &State) -> bool {
    state.a.is_empty() && state.b.is_empty() && base_len(&state.d) == 0 && !state.d.is_empty()
}

#[allow(clippy::too_many_arguments)]
fn dfs_path<H: Fn(&State) -> u32>(
    s: &State,
    g: u32,
    bound: u32,
    last: Option<Move>,
    h_s: u32,
    goal: &State,
    h: &H,
    node_cap: u64,
    nodes: &mut u64,
    on_path: &mut FxHashSet<State>,
    path: &mut Vec<Move>,
) -> R {
    if g + h_s > bound {
        return R::Bound(g + h_s);
    }
    if s == goal {
        return R::Found;
    }
    *nodes += 1;
    if *nodes > node_cap {
        return R::GaveUp;
    }
    let mut best = u32::MAX;
    for (t, mv) in s.successors() {
        if let Some(lm) = last {
            if mv == lm.inv() {
                continue;
            }
        }
        if on_path.contains(&t) {
            continue;
        }
        let h_t = h(&t);
        let ph = h_t.max(h_s.saturating_sub(1));
        if g + 1 + ph > bound {
            best = best.min(g + 1 + ph);
            continue;
        }
        on_path.insert(t.clone());
        path.push(mv);
        let r = dfs_path(
            &t,
            g + 1,
            bound,
            Some(mv),
            ph,
            goal,
            h,
            node_cap,
            nodes,
            on_path,
            path,
        );
        if let R::Found = r {
            return R::Found; // keep the path intact on success
        }
        on_path.remove(&t);
        path.pop();
        match r {
            R::GaveUp => return R::GaveUp,
            R::Bound(rb) => best = best.min(rb),
            R::Found => unreachable!(),
        }
    }
    R::Bound(best)
}

/// Like [`ida_star`] but returns an optimal *move sequence* (the first one found
/// at the optimal bound). `None` if the node cap is hit first.
pub fn ida_star_path<H: Fn(&State) -> u32>(
    start: &State,
    h: &H,
    node_cap: u64,
) -> (Option<Vec<Move>>, u64) {
    let goal = State::goal(start.size());
    let mut nodes = 0u64;
    let mut on_path: FxHashSet<State> = fxset();
    on_path.insert(start.clone());
    let mut bound = h(start);
    loop {
        let mut path: Vec<Move> = Vec::new();
        match dfs_path(
            start,
            0,
            bound,
            None,
            bound,
            &goal,
            h,
            node_cap,
            &mut nodes,
            &mut on_path,
            &mut path,
        ) {
            R::Found => return (Some(path), nodes),
            R::GaveUp => return (None, nodes),
            R::Bound(b) => {
                if b == u32::MAX {
                    return (None, nodes);
                }
                bound = b;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bfs_state_counts_match_reference() {
        // reachable-state counts (Python oracle): n = 2..=8
        let expected = [
            (2, 12),
            (3, 60),
            (4, 360),
            (5, 2520),
            (6, 20160),
            (7, 181440),
        ];
        for (n, count) in expected {
            assert_eq!(bfs_dist(n).len(), count, "state count at n={}", n);
        }
    }

    #[test]
    fn bfs_reversed_is_diameter() {
        for n in 2..=7 {
            let dist = bfs_dist(n);
            let diam = *dist.values().max().unwrap();
            assert_eq!(diam, 4 * (n as u32 - 1)); // 4(n-1) for n<=8
            assert_eq!(dist[&State::reversed_deck(n)], diam);
        }
    }

    #[test]
    fn ida_path_is_optimal_and_sorts() {
        use crate::heuristics::h_joint;
        use crate::util::Rng;
        let mut rng = Rng::new(3);
        for _ in 0..20 {
            let st = State::from_deck(rng.perm(9));
            let (cost, _) = ida_star(&st, &h_joint, 2_000_000);
            let (path, _) = ida_star_path(&st, &h_joint, 2_000_000);
            let path = path.unwrap();
            assert_eq!(path.len() as u32, cost.unwrap());
            assert!(st.applied(&path).is_goal());
        }
    }
}
