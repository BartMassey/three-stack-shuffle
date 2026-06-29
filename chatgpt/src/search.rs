//! Exact reverse breadth-first search, A*, and admissible heuristics.

use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap, HashMap, HashSet, VecDeque};
use std::time::{Duration, Instant};

use crate::{replay, MachineError, Move, Plan, State};

/// One fixed disjoint pattern partition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PatternPartition {
    patterns: Vec<Vec<usize>>,
    label_to_pattern_rank: Vec<Option<(usize, usize)>>,
}

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

    /// Largest exact distance stored in the database.
    #[must_use]
    pub fn maximum_distance(&self) -> usize {
        self.distances.values().copied().max().unwrap_or(0)
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

/// Projects a contiguous value interval to a smaller canonical state.
///
/// Cards outside `low..=high` are deleted. Retained cards keep their stack
/// location and top-to-bottom order, then are relabeled to `1..=k`.
#[must_use]
pub fn project_interval_state(state: &State, low: usize, high: usize) -> State {
    assert!(low >= 1, "interval lower bound must be at least 1");
    assert!(low <= high, "interval must be non-empty");
    let project = |stack: &[usize]| {
        stack
            .iter()
            .filter_map(|&card| {
                if (low..=high).contains(&card) {
                    Some(card - low + 1)
                } else {
                    None
                }
            })
            .collect()
    };
    let projected = State {
        a: project(&state.a),
        d: project(&state.d),
        b: project(&state.b),
    };
    debug_assert_eq!(projected.len(), high - low + 1);
    debug_assert!(projected.validate().is_ok());
    projected
}

/// Projects one arbitrary label pattern to a smaller canonical state.
///
/// The pattern is relabeled by increasing original card value, not by its
/// position in the pattern list. This keeps the projected goal equal to
/// `State::goal(pattern.len())`.
#[must_use]
pub fn project_pattern_state(state: &State, pattern: &[usize]) -> State {
    assert!(!pattern.is_empty(), "pattern must be non-empty");
    let mut labels = pattern.to_vec();
    labels.sort_unstable();
    labels.dedup();
    assert_eq!(labels.len(), pattern.len(), "pattern labels must be unique");
    let max_label = labels.last().copied().unwrap_or(0);
    let mut ranks = vec![0; max_label + 1];
    for (index, &label) in labels.iter().enumerate() {
        assert!(label > 0, "pattern labels must be positive");
        ranks[label] = index + 1;
    }
    let project = |stack: &[usize]| {
        stack
            .iter()
            .filter_map(|&card| ranks.get(card).copied().filter(|&rank| rank > 0))
            .collect()
    };
    let projected = State {
        a: project(&state.a),
        d: project(&state.d),
        b: project(&state.b),
    };
    debug_assert_eq!(projected.len(), pattern.len());
    debug_assert!(projected.validate().is_ok());
    projected
}

/// Balanced contiguous value intervals covering `1..=n`.
///
/// For `n = 52` and `maximum_size = 7`, this returns eight intervals with
/// sizes `7, 7, 7, 7, 6, 6, 6, 6`.
#[must_use]
pub fn balanced_value_intervals(n: usize, maximum_size: usize) -> Vec<(usize, usize)> {
    assert!(maximum_size > 0, "maximum pattern size must be positive");
    if n == 0 {
        return Vec::new();
    }
    let count = n.div_ceil(maximum_size);
    let small = n / count;
    let large_count = n % count;
    let mut low = 1;
    (0..count)
        .map(|index| {
            let size = small + usize::from(index < large_count);
            let high = low + size - 1;
            let interval = (low, high);
            low = high + 1;
            interval
        })
        .collect()
}

impl PatternPartition {
    /// Builds balanced patterns by chunking a permutation in occurrence order.
    ///
    /// Each returned pattern is internally sorted by value for relabeling, but
    /// group membership is determined by the supplied order.
    #[must_use]
    pub fn from_order(order: &[usize], maximum_size: usize) -> Self {
        assert!(maximum_size > 0, "maximum pattern size must be positive");
        if order.is_empty() {
            return Self {
                patterns: Vec::new(),
                label_to_pattern_rank: vec![None],
            };
        }
        let n = order.len();
        let mut seen = vec![false; n + 1];
        for &label in order {
            assert!(
                label > 0 && label <= n && !seen[label],
                "order must be a permutation of 1..=n"
            );
            seen[label] = true;
        }

        let count = n.div_ceil(maximum_size);
        let small = n / count;
        let large_count = n % count;
        let mut offset = 0;
        let mut patterns = Vec::with_capacity(count);
        let mut label_to_pattern_rank = vec![None; n + 1];
        for pattern_index in 0..count {
            let size = small + usize::from(pattern_index < large_count);
            let mut pattern = order[offset..offset + size].to_vec();
            pattern.sort_unstable();
            for (rank, &label) in pattern.iter().enumerate() {
                label_to_pattern_rank[label] = Some((pattern_index, rank + 1));
            }
            patterns.push(pattern);
            offset += size;
        }
        Self {
            patterns,
            label_to_pattern_rank,
        }
    }

    /// Builds balanced patterns from the current top-to-bottom stack order.
    #[must_use]
    pub fn from_state_order(state: &State, maximum_size: usize) -> Self {
        let order = state
            .a
            .iter()
            .chain(&state.d)
            .chain(&state.b)
            .copied()
            .collect::<Vec<_>>();
        Self::from_order(&order, maximum_size)
    }

    /// Pattern label sets, each sorted by original value.
    #[must_use]
    pub fn patterns(&self) -> &[Vec<usize>] {
        &self.patterns
    }

    /// Largest pattern size in this partition.
    #[must_use]
    pub fn maximum_pattern_size(&self) -> usize {
        self.patterns.iter().map(Vec::len).max().unwrap_or_default()
    }

    fn project(&self, state: &State, pattern_index: usize) -> State {
        let project = |stack: &[usize]| {
            stack
                .iter()
                .filter_map(|&card| {
                    let (index, rank) = self.label_to_pattern_rank.get(card).copied().flatten()?;
                    (index == pattern_index).then_some(rank)
                })
                .collect()
        };
        let projected = State {
            a: project(&state.a),
            d: project(&state.d),
            b: project(&state.b),
        };
        debug_assert_eq!(projected.len(), self.patterns[pattern_index].len());
        debug_assert!(projected.validate().is_ok());
        projected
    }
}

/// Exact additive pattern databases keyed by interval size.
#[derive(Clone, Debug)]
pub struct PatternDatabases {
    maximum_size: usize,
    by_size: Vec<ReverseBfs>,
}

impl PatternDatabases {
    /// Builds exact databases for every size `0..=maximum_size`.
    #[must_use]
    pub fn build(maximum_size: usize) -> Self {
        assert!(maximum_size > 0, "maximum pattern size must be positive");
        let by_size = (0..=maximum_size).map(ReverseBfs::build).collect();
        Self {
            maximum_size,
            by_size,
        }
    }

    /// Largest pattern size supported by this context.
    #[must_use]
    pub const fn maximum_size(&self) -> usize {
        self.maximum_size
    }

    /// Exact database for one pattern size.
    #[must_use]
    pub fn database(&self, size: usize) -> Option<&ReverseBfs> {
        self.by_size.get(size)
    }

    /// Exact additive PDB heuristic over balanced contiguous intervals.
    #[must_use]
    pub fn heuristic(&self, state: &State, maximum_size: usize) -> usize {
        assert!(
            maximum_size <= self.maximum_size,
            "requested pattern size exceeds built databases"
        );
        balanced_value_intervals(state.len(), maximum_size)
            .into_iter()
            .map(|(low, high)| {
                let projected = project_interval_state(state, low, high);
                self.by_size[projected.len()]
                    .distance(&projected)
                    .expect("projected state must be present in exact database")
            })
            .sum()
    }

    /// Exact additive PDB heuristic for any fixed disjoint label partition.
    #[must_use]
    pub fn heuristic_for_partition(&self, state: &State, partition: &PatternPartition) -> usize {
        assert!(
            partition.maximum_pattern_size() <= self.maximum_size,
            "partition pattern size exceeds built databases"
        );
        partition
            .patterns
            .iter()
            .enumerate()
            .map(|(index, pattern)| {
                let projected = partition.project(state, index);
                self.by_size[pattern.len()]
                    .distance(&projected)
                    .expect("projected state must be present in exact database")
            })
            .sum()
    }

    /// `max(transport_heuristic, additive PDB)`.
    #[must_use]
    pub fn max_with_transport(&self, state: &State, maximum_size: usize) -> usize {
        transport_heuristic(state).max(self.heuristic(state, maximum_size))
    }

    /// `max(transport_heuristic, additive PDB for a fixed partition)`.
    #[must_use]
    pub fn max_partition_with_transport(
        &self,
        state: &State,
        partition: &PatternPartition,
    ) -> usize {
        transport_heuristic(state).max(self.heuristic_for_partition(state, partition))
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
    astar_with_heuristic(start, transport_heuristic)
}

/// Finds an optimal plan with reopening A* and a caller-supplied heuristic.
pub fn astar_with_heuristic<F>(start: &State, heuristic: F) -> Result<SearchResult, MachineError>
where
    F: Fn(&State) -> usize,
{
    start.validate()?;
    let began = Instant::now();
    let goal = State::goal(start.len());
    let start_h = heuristic(start);
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
                let f = candidate_g + heuristic(&child);
                open.push(Reverse((f, candidate_g, child)));
                stats.max_open = stats.max_open.max(open.len());
            }
        }
    }
    unreachable!("the finite reversible graph always contains the goal")
}

/// Finds an optimal plan with the seven-card additive PDB heuristic.
pub fn astar_pdb(
    start: &State,
    pdb: &PatternDatabases,
    maximum_size: usize,
) -> Result<SearchResult, MachineError> {
    astar_with_heuristic(start, |state| pdb.heuristic(state, maximum_size))
}

/// Finds an optimal plan with `max(transport, additive PDB)`.
pub fn astar_max_transport_pdb(
    start: &State,
    pdb: &PatternDatabases,
    maximum_size: usize,
) -> Result<SearchResult, MachineError> {
    astar_with_heuristic(start, |state| pdb.max_with_transport(state, maximum_size))
}

/// Finds an optimal plan with additive PDB over a fixed arbitrary partition.
pub fn astar_partition_pdb(
    start: &State,
    pdb: &PatternDatabases,
    partition: &PatternPartition,
) -> Result<SearchResult, MachineError> {
    astar_with_heuristic(start, |state| pdb.heuristic_for_partition(state, partition))
}

/// Finds an optimal plan with `max(transport, fixed-partition PDB)`.
pub fn astar_max_transport_partition_pdb(
    start: &State,
    pdb: &PatternDatabases,
    partition: &PatternPartition,
) -> Result<SearchResult, MachineError> {
    astar_with_heuristic(start, |state| {
        pdb.max_partition_with_transport(state, partition)
    })
}

/// Aggregate exhaustive heuristic quality measurements.
#[derive(Clone, Debug, Default)]
pub struct HeuristicMetrics {
    /// States where `h(s) > exact_distance(s)`.
    pub admissibility_failures: usize,
    /// Largest amount by which `h(s) > exact_distance(s)`.
    pub maximum_admissibility_violation: usize,
    /// Largest amount by which `h(s) > 1 + h(s')` on an edge.
    pub maximum_consistency_violation: usize,
    /// Fraction of directed edges violating consistency.
    pub violating_edge_fraction: f64,
    /// Mean ratio `h/d`, excluding the zero-distance goal.
    pub average_distance_fraction: f64,
    /// States where the heuristic equals exact distance.
    pub exact_states: usize,
}

/// Computes admissibility, consistency, and quality measurements.
#[must_use]
pub fn measure_heuristic_with<F>(database: &ReverseBfs, heuristic: F) -> HeuristicMetrics
where
    F: Fn(&State) -> usize,
{
    let mut metrics = HeuristicMetrics::default();
    let mut edges = 0_usize;
    let mut violations = 0_usize;
    let mut ratio_sum = 0.0;
    let mut non_goal = 0_usize;
    for (state, distance) in database.iter() {
        let h = heuristic(state);
        if h > distance {
            metrics.admissibility_failures += 1;
            metrics.maximum_admissibility_violation =
                metrics.maximum_admissibility_violation.max(h - distance);
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
            let child_h = heuristic(&child);
            if h > child_h + 1 {
                violations += 1;
                metrics.maximum_consistency_violation =
                    metrics.maximum_consistency_violation.max(h - child_h - 1);
            }
        }
    }
    metrics.violating_edge_fraction = if edges == 0 {
        0.0
    } else {
        violations as f64 / edges as f64
    };
    metrics.average_distance_fraction = if non_goal == 0 {
        0.0
    } else {
        ratio_sum / non_goal as f64
    };
    metrics
}

/// Verifies admissibility and computes consistency/quality measurements.
pub fn validate_heuristic_with<F>(
    database: &ReverseBfs,
    heuristic: F,
) -> Result<HeuristicMetrics, String>
where
    F: Fn(&State) -> usize,
{
    let metrics = measure_heuristic_with(database, heuristic);
    if metrics.admissibility_failures == 0 {
        Ok(metrics)
    } else {
        Err(format!(
            "inadmissible heuristic: {} failures, maximum violation {}",
            metrics.admissibility_failures, metrics.maximum_admissibility_violation
        ))
    }
}

/// Verifies admissibility and computes measurements for the transport heuristic.
pub fn validate_heuristic(database: &ReverseBfs) -> Result<HeuristicMetrics, String> {
    validate_heuristic_with(database, transport_heuristic)
}

/// Verifies the additive PDB heuristic.
pub fn validate_pdb_heuristic(
    database: &ReverseBfs,
    pdb: &PatternDatabases,
    maximum_size: usize,
) -> Result<HeuristicMetrics, String> {
    validate_heuristic_with(database, |state| pdb.heuristic(state, maximum_size))
}

/// Verifies `max(transport, additive PDB)`.
pub fn validate_max_transport_pdb_heuristic(
    database: &ReverseBfs,
    pdb: &PatternDatabases,
    maximum_size: usize,
) -> Result<HeuristicMetrics, String> {
    validate_heuristic_with(database, |state| {
        pdb.max_with_transport(state, maximum_size)
    })
}

/// Verifies the additive PDB heuristic for a fixed arbitrary partition.
pub fn validate_partition_pdb_heuristic(
    database: &ReverseBfs,
    pdb: &PatternDatabases,
    partition: &PatternPartition,
) -> Result<HeuristicMetrics, String> {
    validate_heuristic_with(database, |state| {
        pdb.heuristic_for_partition(state, partition)
    })
}

/// Verifies `max(transport, fixed-partition PDB)`.
pub fn validate_max_transport_partition_pdb_heuristic(
    database: &ReverseBfs,
    pdb: &PatternDatabases,
    partition: &PatternPartition,
) -> Result<HeuristicMetrics, String> {
    validate_heuristic_with(database, |state| {
        pdb.max_partition_with_transport(state, partition)
    })
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
    fn interval_projection_preserves_locations_order_and_relabels() {
        let state = State::new(vec![8, 4, 1], vec![7, 3, 6], vec![5, 2]).unwrap();
        let projected = project_interval_state(&state, 3, 6);
        assert_eq!(
            projected,
            State {
                a: vec![2],
                d: vec![1, 4],
                b: vec![3],
            }
        );
        projected.validate().unwrap();
    }

    #[test]
    fn arbitrary_pattern_projection_relabels_by_value_rank() {
        let state = State::new(vec![8, 4, 1], vec![7, 3, 6], vec![5, 2]).unwrap();
        let projected = project_pattern_state(&state, &[7, 2, 4, 6]);
        assert_eq!(
            projected,
            State {
                a: vec![2],
                d: vec![4, 3],
                b: vec![1],
            }
        );
        projected.validate().unwrap();
    }

    #[test]
    fn balanced_intervals_cover_exactly_with_near_equal_sizes() {
        assert_eq!(
            balanced_value_intervals(52, 7)
                .into_iter()
                .map(|(low, high)| high - low + 1)
                .collect::<Vec<_>>(),
            vec![7, 7, 7, 7, 6, 6, 6, 6]
        );

        for n in 0..=80 {
            for maximum_size in 1..=10 {
                let intervals = balanced_value_intervals(n, maximum_size);
                if n == 0 {
                    assert!(intervals.is_empty());
                    continue;
                }
                assert_eq!(intervals.first().map(|&(low, _)| low), Some(1));
                assert_eq!(intervals.last().map(|&(_, high)| high), Some(n));
                let sizes = intervals
                    .iter()
                    .map(|&(low, high)| {
                        assert!(low <= high);
                        high - low + 1
                    })
                    .collect::<Vec<_>>();
                assert!(sizes.iter().all(|&size| size <= maximum_size));
                assert!(sizes.iter().max().unwrap() - sizes.iter().min().unwrap() <= 1);
                for pair in intervals.windows(2) {
                    assert_eq!(pair[0].1 + 1, pair[1].0);
                }
            }
        }
    }

    #[test]
    fn order_partition_chunks_by_occurrence_and_relabels_by_value() {
        let partition = PatternPartition::from_order(&[8, 1, 7, 2, 6, 3, 5, 4], 3);
        assert_eq!(
            partition.patterns(),
            &[vec![1, 7, 8], vec![2, 3, 6], vec![4, 5]]
        );

        let state = State::initial(&[8, 1, 7, 2, 6, 3, 5, 4]).unwrap();
        assert_eq!(
            partition.project(&state, 0),
            State {
                a: vec![],
                d: vec![3, 1, 2],
                b: vec![],
            }
        );
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
    fn pdb_is_exact_when_the_whole_state_is_one_pattern() {
        let bfs = ReverseBfs::build(7);
        let pdb = PatternDatabases::build(7);
        for (state, distance) in bfs.iter() {
            assert_eq!(pdb.heuristic(state, 7), distance, "state {state:?}");
        }
    }

    #[test]
    fn additive_pdb_and_max_are_exhaustively_admissible() {
        let bfs = ReverseBfs::build(6);
        let pdb = PatternDatabases::build(3);
        let pdb_metrics = validate_pdb_heuristic(&bfs, &pdb, 3).unwrap();
        assert_eq!(pdb_metrics.maximum_consistency_violation, 0);
        assert!(pdb_metrics.exact_states > 1);

        let max_metrics = validate_max_transport_pdb_heuristic(&bfs, &pdb, 3).unwrap();
        assert!(max_metrics.exact_states >= pdb_metrics.exact_states);
    }

    #[test]
    fn initial_order_partition_pdb_is_exhaustively_admissible_and_consistent() {
        let bfs = ReverseBfs::build(6);
        let pdb = PatternDatabases::build(3);
        let partition = PatternPartition::from_order(&[6, 1, 5, 2, 4, 3], 3);
        let pdb_metrics = validate_partition_pdb_heuristic(&bfs, &pdb, &partition).unwrap();
        assert_eq!(pdb_metrics.maximum_consistency_violation, 0);

        let max_metrics =
            validate_max_transport_partition_pdb_heuristic(&bfs, &pdb, &partition).unwrap();
        assert!(max_metrics.exact_states >= pdb_metrics.exact_states);
    }

    #[test]
    fn known_pdb_states_compare_transport_pdb_and_max() {
        let pdb = PatternDatabases::build(7);
        assert_eq!(pdb.heuristic(&State::goal(52), 7), 0);
        assert_eq!(pdb.max_with_transport(&State::goal(52), 7), 0);

        let states = [
            State::initial(&(1..=52).rev().collect::<Vec<_>>()).unwrap(),
            State::initial(&[3, 1, 4, 2, 6, 5]).unwrap(),
            State::new(vec![8, 3], vec![1, 7, 4], vec![6, 2, 5]).unwrap(),
        ];
        for state in states {
            let transport = transport_heuristic(&state);
            let pdb_value = pdb.heuristic(&state, 7);
            let combined = pdb.max_with_transport(&state, 7);
            assert_eq!(combined, transport.max(pdb_value));
        }
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
