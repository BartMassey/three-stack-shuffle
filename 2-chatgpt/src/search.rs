//! Exact reverse breadth-first search, A*, and the transport heuristic.

use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap, HashMap, HashSet, VecDeque};
use std::time::{Duration, Instant};

use crate::{replay, MachineError, Move, Plan, State};

/// Complete exact-distance database built outward from a goal state.
#[derive(Clone, Debug)]
pub struct ReverseBfs {
    n: usize,
    distances: HashMap<State, usize>,
    toward_goal: HashMap<State, Move>,
}

impl ReverseBfs {
    /// Enumerates the complete state graph for `n` cards.
    #[must_use]
    pub fn build(n: usize) -> Self {
        let goal = State::goal(n);
        let mut distances = HashMap::from([(goal.clone(), 0)]);
        let mut toward_goal = HashMap::new();
        let mut queue = VecDeque::from([goal]);
        while let Some(state) = queue.pop_front() {
            let distance = distances[&state];
            for (child, movement) in state.neighbors() {
                if !distances.contains_key(&child) {
                    // The graph is undirected; this move returns the newly
                    // discovered child to its BFS parent.
                    toward_goal.insert(child.clone(), movement.inverse());
                    distances.insert(child.clone(), distance + 1);
                    queue.push_back(child);
                }
            }
        }
        Self {
            n,
            distances,
            toward_goal,
        }
    }

    /// Number of states in the database.
    #[must_use]
    pub fn len(&self) -> usize {
        self.distances.len()
    }

    /// Returns whether no states were enumerated.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.distances.is_empty()
    }

    /// Returns the exact goal distance of a state, if it belongs to this graph.
    #[must_use]
    pub fn distance(&self, state: &State) -> Option<usize> {
        self.distances.get(state).copied()
    }

    /// Iterates over every state and exact distance.
    pub fn iter(&self) -> impl Iterator<Item = (&State, usize)> {
        self.distances
            .iter()
            .map(|(state, &distance)| (state, distance))
    }

    /// Reconstructs an optimal plan from `state` to the goal.
    #[must_use]
    pub fn plan(&self, state: &State) -> Option<Plan> {
        if !self.distances.contains_key(state) {
            return None;
        }
        let mut current = state.clone();
        let mut plan = Vec::with_capacity(self.distances[&current]);
        while current != State::goal(self.n) {
            let movement = self.toward_goal[&current];
            current = replay(&current, &[movement]).ok()?;
            plan.push(movement);
        }
        Some(plan)
    }
}

/// Returns the longest bottom suffix of D already equal to the goal suffix.
#[must_use]
pub fn frozen_suffix_length(state: &State) -> usize {
    let mut expected = state.len();
    let mut frozen = 0;
    for &card in state.d.iter().rev() {
        if card != expected {
            break;
        }
        frozen += 1;
        expected = expected.saturating_sub(1);
    }
    frozen
}

/// Computes longest decreasing subsequence length in quadratic time.
#[must_use]
pub fn lds_length(values: &[usize]) -> usize {
    let mut best = vec![1; values.len()];
    for i in 0..values.len() {
        for j in 0..i {
            if values[j] > values[i] {
                best[i] = best[i].max(best[j] + 1);
            }
        }
    }
    best.into_iter().max().unwrap_or(0)
}

/// Maximum subsequence size coverable by at most two increasing sequences.
#[must_use]
pub fn max_two_increasing_cover(values: &[usize]) -> usize {
    let mut states = BTreeMap::from([((0, 0), 0_usize)]);
    for &value in values {
        // This clone is the required snapshot: updates made for this card must
        // not be revisited until the next card.
        let mut next = states.clone();
        for (&(u, v), &count) in &states {
            if value > u {
                let tails = if value <= v { (value, v) } else { (v, value) };
                next.entry(tails)
                    .and_modify(|old| *old = (*old).max(count + 1))
                    .or_insert(count + 1);
            }
            if value > v {
                next.entry((u, value))
                    .and_modify(|old| *old = (*old).max(count + 1))
                    .or_insert(count + 1);
            }
        }
        states = next;
    }
    states.into_values().max().unwrap_or(0)
}

/// Admissible, intentionally inconsistent transport lower bound.
#[must_use]
pub fn transport_heuristic(state: &State) -> usize {
    let frozen = frozen_suffix_length(state);
    let active_len = state.d.len() - frozen;
    let active = &state.d[..active_len];
    let base = 2 * active_len + state.a.len() + state.b.len();
    let endpoint_extra =
        2 * (state.a.len() - lds_length(&state.a)) + 2 * (state.b.len() - lds_length(&state.b));
    let center_extra = 2 * (active_len - max_two_increasing_cover(active));
    base + endpoint_extra + center_extra
}

/// Instrumentation from one A* invocation.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SearchStats {
    /// Neighbor states generated, including duplicates.
    pub generated: usize,
    /// Non-stale queue entries expanded.
    pub expanded: usize,
    /// Improvements to states that had already been expanded.
    pub reopened: usize,
    /// Superseded queue entries discarded.
    pub stale: usize,
    /// Maximum number of queued entries.
    pub max_open: usize,
    /// Wall-clock search duration.
    pub elapsed: Duration,
}

/// Optimal A* result and reproducibility metrics.
#[derive(Clone, Debug)]
pub struct SearchResult {
    /// Optimal primitive move plan.
    pub plan: Plan,
    /// Heuristic value at the initial state.
    pub start_heuristic: usize,
    /// Search instrumentation.
    pub stats: SearchStats,
}

/// Finds an optimal plan with reopening A* and the transport heuristic.
pub fn astar(start: &State) -> Result<SearchResult, MachineError> {
    start.validate()?;
    let began = Instant::now();
    let goal = State::goal(start.len());
    let start_h = transport_heuristic(start);
    // Reverse makes this max-heap a deterministic min-heap over (f, g, state).
    let mut open = BinaryHeap::from([Reverse((start_h, 0_usize, start.clone()))]);
    let mut best_g = HashMap::from([(start.clone(), 0_usize)]);
    let mut parents: HashMap<State, (State, Move)> = HashMap::new();
    let mut expanded = HashSet::new();
    let mut stats = SearchStats {
        max_open: 1,
        ..SearchStats::default()
    };

    while let Some(Reverse((_f, queued_g, state))) = open.pop() {
        if best_g.get(&state) != Some(&queued_g) {
            stats.stale += 1;
            continue;
        }
        if state == goal {
            let mut current = goal;
            let mut reverse_plan = Vec::with_capacity(queued_g);
            while current != *start {
                let (parent, movement) = parents[&current].clone();
                reverse_plan.push(movement);
                current = parent;
            }
            reverse_plan.reverse();
            stats.elapsed = began.elapsed();
            return Ok(SearchResult {
                plan: reverse_plan,
                start_heuristic: start_h,
                stats,
            });
        }
        expanded.insert(state.clone());
        stats.expanded += 1;
        for (child, movement) in state.neighbors() {
            stats.generated += 1;
            let candidate_g = queued_g + 1;
            if best_g.get(&child).is_none_or(|&known| candidate_g < known) {
                if expanded.contains(&child) {
                    stats.reopened += 1;
                }
                best_g.insert(child.clone(), candidate_g);
                parents.insert(child.clone(), (state.clone(), movement));
                let f = candidate_g + transport_heuristic(&child);
                open.push(Reverse((f, candidate_g, child)));
                stats.max_open = stats.max_open.max(open.len());
            }
        }
    }
    unreachable!("the finite reversible graph always contains the goal")
}

/// Aggregate exhaustive heuristic quality measurements.
#[derive(Clone, Debug, Default)]
pub struct HeuristicMetrics {
    /// Largest amount by which `h(s) > 1 + h(s')` on an edge.
    pub maximum_consistency_violation: usize,
    /// Fraction of directed edges violating consistency.
    pub violating_edge_fraction: f64,
    /// Mean ratio `h/d`, excluding the zero-distance goal.
    pub average_distance_fraction: f64,
    /// States where the heuristic equals exact distance.
    pub exact_states: usize,
}

/// Verifies admissibility and computes consistency/quality measurements.
pub fn validate_heuristic(database: &ReverseBfs) -> Result<HeuristicMetrics, String> {
    let mut metrics = HeuristicMetrics::default();
    let mut edges = 0_usize;
    let mut violations = 0_usize;
    let mut ratio_sum = 0.0;
    let mut non_goal = 0_usize;
    for (state, distance) in database.iter() {
        let h = transport_heuristic(state);
        if h > distance {
            return Err(format!(
                "inadmissible state {state:?}: heuristic {h}, distance {distance}"
            ));
        }
        if h == distance {
            metrics.exact_states += 1;
        }
        if distance > 0 {
            ratio_sum += h as f64 / distance as f64;
            non_goal += 1;
        }
        for (child, _) in state.neighbors() {
            edges += 1;
            let child_h = transport_heuristic(&child);
            if h > child_h + 1 {
                violations += 1;
                metrics.maximum_consistency_violation =
                    metrics.maximum_consistency_violation.max(h - child_h - 1);
            }
        }
    }
    metrics.violating_edge_fraction = violations as f64 / edges as f64;
    metrics.average_distance_fraction = ratio_sum / non_goal as f64;
    Ok(metrics)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_count_and_edge_reversal() {
        let bfs = ReverseBfs::build(6);
        assert_eq!(bfs.len(), 720 * 28);
        for (state, _) in bfs.iter() {
            for (child, movement) in state.neighbors() {
                assert_eq!(replay(&child, &[movement.inverse()]).unwrap(), *state);
            }
        }
    }

    #[test]
    fn transport_regressions() {
        assert_eq!(transport_heuristic(&State::goal(6)), 0);
        let reversed = State::initial(&(1..=52).rev().collect::<Vec<_>>()).unwrap();
        assert_eq!(transport_heuristic(&reversed), 204);
        let state = State::initial(&[3, 2, 1]).unwrap();
        assert_eq!(transport_heuristic(&state), 8);
        let child = replay(&state, &[Move::DtoA]).unwrap();
        assert_eq!(transport_heuristic(&child), 5);
    }

    #[test]
    fn heuristic_is_exhaustively_admissible() {
        let bfs = ReverseBfs::build(6);
        let metrics = validate_heuristic(&bfs).unwrap();
        assert_eq!(metrics.maximum_consistency_violation, 2);
        assert!(metrics.exact_states > 1);
    }

    #[test]
    fn astar_matches_bfs_for_every_initial_permutation_through_six() {
        let bfs = ReverseBfs::build(6);
        for (state, distance) in bfs
            .iter()
            .filter(|(state, _)| state.a.is_empty() && state.b.is_empty())
        {
            let result = astar(state).unwrap();
            assert_eq!(result.plan.len(), distance, "state {state:?}");
            assert_eq!(replay(state, &result.plan).unwrap(), State::goal(6));
        }
    }
}
