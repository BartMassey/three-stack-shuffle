//! Constructive sorting algorithms from `ALGORITHMS.md`.
//!
//! Every implementation drives [`Machine`]; none mutates a
//! stack directly. Experimental algorithms are explicitly marked in
//! [`Algorithm`] and use certified fallbacks when their pure phase rule is not
//! established by the specification.

use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap, HashMap, HashSet};
use std::mem::size_of;
use std::time::Instant;

use crate::macros::{move_cards, reverse_d, reverse_d_to_endpoint};
use crate::search::transport_heuristic;
use crate::{Machine, MachineError, Move, Plan, StackId, State};

/// A constructive sorting algorithm.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Algorithm {
    /// Sweep-based selection sort.
    Selection,
    /// Selection sort that turns around after every selected card.
    AdaptiveSelection,
    /// Gene Welborn's selection sort that stages consecutive future targets.
    LookaheadSelection,
    /// Gene Welborn's lookahead selection over `2 * k` balanced value buckets.
    TwoKPartitionLookaheadSelection(usize),
    /// Exhaustive capture-mask rollout over `2 * k` balanced value buckets.
    RolloutTwoKPartitionLookaheadSelectionExperimental(usize),
    /// Incrementally memoized equivalent of exhaustive rollout.
    IncrementalRhlTwoKPartitionLookaheadSelectionExperimental(usize),
    /// Depth-limited binary-decision receding-horizon rollout.
    DepthLimitedRhlTwoKPartitionLookaheadSelectionExperimental(usize, usize),
    /// Receding-horizon rollout over target blocks built by one parked reversal.
    TargetBlockRolloutSelectionExperimental,
    /// Balanced value partitioning with target-block rollout in each leaf.
    TargetBlockRolloutTwoKPartitionSelectionExperimental(usize),
    /// Optimal A* leaf extraction over `2 * k` balanced value buckets.
    TwoKPartitioningPerfectSelection(usize),
    /// Adaptive selection preceded by one binary value partition.
    BinaryPresortAdaptiveSelection,
    /// Literal top-down merge sort.
    Merge,
    /// Most-significant-bit value-range radix sort.
    MsbRadix,
    /// Stable least-significant-bit radix sort.
    LsbRadix,
    /// Full-pass natural merge sort.
    Natural,
    /// Optimal alphabetic merge tree over the input's ascending runs.
    HuTuckerNaturalMerge,
    /// Safe signed-natural hybrid (experimental).
    SignedNaturalExperimental,
    /// Two-increasing-subsequence split-merge sort.
    SplitMerge,
    /// Safe reversing split-merge variant (experimental).
    ReversingSplitMergeExperimental,
}

impl Algorithm {
    /// All implemented algorithms and standard parameter configurations in
    /// stable display order.
    pub const ALL: [Self; 16] = [
        Self::Selection,
        Self::AdaptiveSelection,
        Self::LookaheadSelection,
        Self::TwoKPartitionLookaheadSelection(1),
        Self::TwoKPartitionLookaheadSelection(2),
        Self::TwoKPartitionLookaheadSelection(3),
        Self::TwoKPartitionLookaheadSelection(4),
        Self::BinaryPresortAdaptiveSelection,
        Self::Merge,
        Self::MsbRadix,
        Self::LsbRadix,
        Self::Natural,
        Self::HuTuckerNaturalMerge,
        Self::SignedNaturalExperimental,
        Self::SplitMerge,
        Self::ReversingSplitMergeExperimental,
    ];

    /// Returns the stable command-line name.
    #[must_use]
    pub fn name(self) -> String {
        match self {
            Self::Selection => "selection".into(),
            Self::AdaptiveSelection => "adaptive-selection".into(),
            Self::LookaheadSelection => "lookahead-selection".into(),
            Self::TwoKPartitionLookaheadSelection(k) => {
                format!("2k-partition-lookahead-selection:{k}")
            }
            Self::RolloutTwoKPartitionLookaheadSelectionExperimental(k) => {
                format!("rollout-2k-partition-lookahead-selection-experimental:{k}")
            }
            Self::IncrementalRhlTwoKPartitionLookaheadSelectionExperimental(k) => {
                format!("incremental-rhl-2k-partition-lookahead-selection-experimental:{k}")
            }
            Self::DepthLimitedRhlTwoKPartitionLookaheadSelectionExperimental(k, depth) => {
                format!(
                    "depth-limited-rhl-2k-partition-lookahead-selection-experimental:{k}:{depth}"
                )
            }
            Self::TargetBlockRolloutSelectionExperimental => {
                "target-block-rollout-selection-experimental".into()
            }
            Self::TargetBlockRolloutTwoKPartitionSelectionExperimental(k) => {
                format!("target-block-rollout-2k-partition-selection-experimental:{k}")
            }
            Self::TwoKPartitioningPerfectSelection(k) => {
                format!("2k-partitioning-perfect-selection:{k}")
            }
            Self::BinaryPresortAdaptiveSelection => "binary-presort-adaptive-selection".into(),
            Self::Merge => "merge".into(),
            Self::MsbRadix => "msb-radix".into(),
            Self::LsbRadix => "lsb-radix".into(),
            Self::Natural => "natural".into(),
            Self::HuTuckerNaturalMerge => "hu-tucker-natural-merge".into(),
            Self::SignedNaturalExperimental => "signed-natural-experimental".into(),
            Self::SplitMerge => "split-merge".into(),
            Self::ReversingSplitMergeExperimental => "reversing-split-merge-experimental".into(),
        }
    }

    /// Returns whether the algorithm is experimental.
    #[must_use]
    pub const fn is_experimental(self) -> bool {
        matches!(
            self,
            Self::RolloutTwoKPartitionLookaheadSelectionExperimental(_)
                | Self::IncrementalRhlTwoKPartitionLookaheadSelectionExperimental(_)
                | Self::DepthLimitedRhlTwoKPartitionLookaheadSelectionExperimental(_, _)
                | Self::TargetBlockRolloutSelectionExperimental
                | Self::TargetBlockRolloutTwoKPartitionSelectionExperimental(_)
                | Self::TwoKPartitioningPerfectSelection(_)
                | Self::SignedNaturalExperimental
                | Self::ReversingSplitMergeExperimental
        )
    }

    /// Parses a stable command-line name.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        if let Some(k) =
            name.strip_prefix("incremental-rhl-2k-partition-lookahead-selection-experimental:")
        {
            return k
                .parse::<usize>()
                .ok()
                .filter(|&k| k > 0)
                .map(Self::IncrementalRhlTwoKPartitionLookaheadSelectionExperimental);
        }
        if let Some(rest) =
            name.strip_prefix("depth-limited-rhl-2k-partition-lookahead-selection-experimental:")
        {
            let (k, depth) = rest.split_once(':')?;
            return Some(
                Self::DepthLimitedRhlTwoKPartitionLookaheadSelectionExperimental(
                    k.parse::<usize>().ok().filter(|&k| k > 0)?,
                    depth.parse::<usize>().ok()?,
                ),
            );
        }
        if let Some(k) = name.strip_prefix("rollout-2k-partition-lookahead-selection-experimental:")
        {
            return k
                .parse::<usize>()
                .ok()
                .filter(|&k| k > 0)
                .map(Self::RolloutTwoKPartitionLookaheadSelectionExperimental);
        }
        if let Some(k) = name.strip_prefix("2k-partitioning-perfect-selection:") {
            return k
                .parse::<usize>()
                .ok()
                .filter(|&k| k > 0)
                .map(Self::TwoKPartitioningPerfectSelection);
        }
        if let Some(k) = name.strip_prefix("2k-partition-lookahead-selection:") {
            return k
                .parse::<usize>()
                .ok()
                .filter(|&k| k > 0)
                .map(Self::TwoKPartitionLookaheadSelection);
        }
        if let Some(k) =
            name.strip_prefix("target-block-rollout-2k-partition-selection-experimental:")
        {
            return k
                .parse::<usize>()
                .ok()
                .filter(|&k| k > 0)
                .map(Self::TargetBlockRolloutTwoKPartitionSelectionExperimental);
        }
        if name == "target-block-rollout-selection-experimental" {
            return Some(Self::TargetBlockRolloutSelectionExperimental);
        }
        Self::ALL
            .into_iter()
            .find(|algorithm| algorithm.name() == name)
    }
}

/// Algorithm-specific counters collected without affecting move cost.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SortStats {
    /// Complete selection sweeps.
    pub sweeps: usize,
    /// Endpoint-to-endpoint card bypasses.
    pub bypasses: usize,
    /// Merge or distribution phases.
    pub phases: usize,
    /// Initial ascending run count, where applicable.
    pub initial_runs: usize,
    /// Explicit reversal operations.
    pub reversals: usize,
    /// Incremental-RHL planning measurements, when applicable.
    pub incremental_rhl: IncrementalRhlStats,
    /// Depth-limited-RHL planning measurements, when applicable.
    pub depth_limited_rhl: DepthLimitedRhlStats,
    /// Consecutive-target block-DP planning measurements, when applicable.
    pub target_block_dp: TargetBlockDpStats,
}

/// Planning counters for incremental receding-horizon lookahead.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct IncrementalRhlStats {
    /// Distinct mask outcomes evaluated after the bottommost-bit quotient.
    pub masks_visited: usize,
    /// Distinct unnormalized algebraic successor states constructed.
    pub distinct_algebraic_successors: usize,
    /// Distinct successors after forced-target and endpoint normalization.
    pub distinct_normalized_successors: usize,
    /// Persistent base-policy cache hits.
    pub base_cache_hits: usize,
    /// Persistent base-policy cache misses.
    pub base_cache_misses: usize,
    /// Number of base-policy states stored.
    pub base_states_stored: usize,
    /// Exposed deterministic targets removed during base-policy evaluation.
    pub forced_targets_removed: usize,
    /// Estimated peak bytes occupied by planner hash tables and owned vectors.
    pub estimated_peak_memory_bytes: usize,
    /// Total planning time, excluding execution of committed masks.
    pub planning_nanos: u128,
    /// Number of nontrivial target decisions planned.
    pub planning_targets: usize,
    /// Number of leaf buckets planned.
    pub planning_buckets: usize,
}

/// Planning counters for depth-limited receding-horizon lookahead.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DepthLimitedRhlStats {
    /// Configured binary-decision search depth.
    pub depth: usize,
    /// Binary decision nodes expanded by the planner.
    pub binary_nodes_expanded: usize,
    /// Greedy terminal frontier evaluations requested.
    pub frontier_evaluations: usize,
    /// Greedy terminal cache hits.
    pub greedy_cache_hits: usize,
    /// Greedy terminal cache misses.
    pub greedy_cache_misses: usize,
    /// Deterministic suffix-cache hits inside optimized greedy terminal evaluation.
    pub suffix_cache_hits: usize,
    /// Deterministic suffix-cache misses inside optimized greedy terminal evaluation.
    pub suffix_cache_misses: usize,
    /// Number of deterministic suffix states stored.
    pub suffix_states_stored: usize,
    /// Forced targets removed during deterministic suffix evaluation.
    pub suffix_forced_targets_removed: usize,
    /// Depth-value cache hits.
    pub depth_cache_hits: usize,
    /// Depth-value cache misses.
    pub depth_cache_misses: usize,
    /// Nodes retained after rerooting. Zero in the baseline memoized planner.
    pub nodes_retained_after_rerooting: usize,
    /// New nodes added per real decision. Equal to expanded nodes per decision in the baseline.
    pub new_nodes_added: usize,
    /// Estimated peak bytes occupied by planner hash tables and owned vectors.
    pub estimated_peak_memory_bytes: usize,
    /// Total planning time, excluding execution of committed decisions.
    pub planning_nanos: u128,
    /// Number of real binary decisions planned.
    pub planning_decisions: usize,
    /// Number of leaf buckets planned.
    pub planning_buckets: usize,
}

/// Planning counters for consecutive-target block DP and rollout.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TargetBlockDpStats {
    /// Canonical endpoint states whose values were computed.
    pub states: usize,
    /// Candidate consecutive-target batch transitions evaluated.
    pub transitions: usize,
    /// Largest number of batch lengths considered at one state.
    pub max_candidates: usize,
    /// Memoized value lookups reused.
    pub cache_hits: usize,
    /// Exposed targets removed without a branching decision.
    pub forced_targets: usize,
}

/// Replayable isolated target-block planning result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetBlockDpResult {
    /// Primitive legal move sequence from the supplied A-stack permutation.
    pub plan: Plan,
    /// Dynamic-programming state and transition counts.
    pub stats: TargetBlockDpStats,
}

/// A fully replayable result from a constructive algorithm.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SortResult {
    /// Selected algorithm.
    pub algorithm: Algorithm,
    /// Primitive legal move sequence.
    pub plan: Plan,
    /// Free bookkeeping counters.
    pub stats: SortStats,
}

impl SortResult {
    /// Returns the cost, exactly equal to the primitive plan length.
    #[must_use]
    pub fn cost(&self) -> usize {
        self.plan.len()
    }
}

/// Runs a constructive sorter after validating the input permutation.
pub fn solve(algorithm: Algorithm, deck: &[usize]) -> Result<SortResult, MachineError> {
    if matches!(
        algorithm,
        Algorithm::TwoKPartitionLookaheadSelection(0)
            | Algorithm::RolloutTwoKPartitionLookaheadSelectionExperimental(0)
            | Algorithm::IncrementalRhlTwoKPartitionLookaheadSelectionExperimental(0)
            | Algorithm::DepthLimitedRhlTwoKPartitionLookaheadSelectionExperimental(0, _)
            | Algorithm::TargetBlockRolloutTwoKPartitionSelectionExperimental(0)
            | Algorithm::TwoKPartitioningPerfectSelection(0)
    ) {
        return Err(MachineError::InvalidAlgorithmParameter(
            "2k-partition lookahead selection requires k >= 1",
        ));
    }
    // The free sorted-input check is shared by every implementation.
    let initial = State::initial(deck)?;
    if initial == State::goal(deck.len()) {
        return Ok(SortResult {
            algorithm,
            plan: Vec::new(),
            stats: SortStats {
                initial_runs: usize::from(!deck.is_empty()),
                ..SortStats::default()
            },
        });
    }

    match algorithm {
        Algorithm::Selection => selection(deck, algorithm),
        Algorithm::AdaptiveSelection => adaptive_selection(deck, algorithm),
        Algorithm::LookaheadSelection => lookahead_selection(deck, algorithm),
        Algorithm::TwoKPartitionLookaheadSelection(k) => {
            let buckets = k
                .checked_mul(2)
                .ok_or(MachineError::InvalidAlgorithmParameter(
                    "2k-partition lookahead selection bucket count overflowed",
                ))?;
            partition_lookahead_selection(deck, algorithm, buckets, LeafSelection::Consecutive)
        }
        Algorithm::RolloutTwoKPartitionLookaheadSelectionExperimental(k) => {
            let buckets = k
                .checked_mul(2)
                .ok_or(MachineError::InvalidAlgorithmParameter(
                    "rollout 2k-partition lookahead selection bucket count overflowed",
                ))?;
            partition_lookahead_selection(deck, algorithm, buckets, LeafSelection::Rollout)
        }
        Algorithm::IncrementalRhlTwoKPartitionLookaheadSelectionExperimental(k) => {
            let buckets = k
                .checked_mul(2)
                .ok_or(MachineError::InvalidAlgorithmParameter(
                    "incremental RHL 2k-partition lookahead selection bucket count overflowed",
                ))?;
            partition_lookahead_selection(deck, algorithm, buckets, LeafSelection::IncrementalRhl)
        }
        Algorithm::DepthLimitedRhlTwoKPartitionLookaheadSelectionExperimental(k, depth) => {
            let buckets = k
                .checked_mul(2)
                .ok_or(MachineError::InvalidAlgorithmParameter(
                    "depth-limited RHL 2k-partition lookahead selection bucket count overflowed",
                ))?;
            partition_lookahead_selection(
                deck,
                algorithm,
                buckets,
                LeafSelection::DepthLimitedRhl { depth },
            )
        }
        Algorithm::TargetBlockRolloutSelectionExperimental => {
            target_block_rollout_selection(deck, algorithm)
        }
        Algorithm::TargetBlockRolloutTwoKPartitionSelectionExperimental(k) => {
            let buckets = k
                .checked_mul(2)
                .ok_or(MachineError::InvalidAlgorithmParameter(
                    "target-block 2k-partition bucket count overflowed",
                ))?;
            partition_lookahead_selection(deck, algorithm, buckets, LeafSelection::TargetBlock)
        }
        Algorithm::TwoKPartitioningPerfectSelection(k) => {
            let buckets = k
                .checked_mul(2)
                .ok_or(MachineError::InvalidAlgorithmParameter(
                    "2k-partitioning perfect selection bucket count overflowed",
                ))?;
            partition_lookahead_selection(deck, algorithm, buckets, LeafSelection::Perfect)
        }
        Algorithm::BinaryPresortAdaptiveSelection => binary_presort(deck, algorithm),
        Algorithm::Merge => merge_sort(deck, algorithm),
        Algorithm::MsbRadix => msb_radix(deck, algorithm),
        Algorithm::LsbRadix => lsb_radix(deck, algorithm),
        Algorithm::Natural => natural_sort(deck, algorithm),
        Algorithm::HuTuckerNaturalMerge => hu_tucker_natural_merge(deck, algorithm),
        Algorithm::SignedNaturalExperimental => signed_natural(deck, algorithm),
        Algorithm::SplitMerge => split_merge(deck, algorithm),
        Algorithm::ReversingSplitMergeExperimental => reversing_split_merge(deck, algorithm),
    }
}

fn active_prefix(deck: &[usize]) -> usize {
    let mut m = deck.len();
    while m > 0 && deck[m - 1] == m {
        m -= 1;
    }
    m
}

fn finish(machine: &mut Machine, algorithm: Algorithm, stats: SortStats) -> SortResult {
    SortResult {
        algorithm,
        plan: machine.take_plan(),
        stats,
    }
}

fn selection(deck: &[usize], algorithm: Algorithm) -> Result<SortResult, MachineError> {
    let mut machine = Machine::new(deck)?;
    let m = active_prefix(deck);
    move_cards(&mut machine, m, StackId::D, StackId::A)?;
    let mut source = StackId::A;
    let mut destination = StackId::B;
    let mut next = m;
    let mut stats = SortStats::default();
    while next > 0 {
        stats.sweeps += 1;
        while match source {
            StackId::A => !machine.state().a.is_empty(),
            StackId::B => !machine.state().b.is_empty(),
            StackId::D => false,
        } {
            let top = match source {
                StackId::A => machine.state().a[0],
                StackId::B => machine.state().b[0],
                StackId::D => unreachable!(),
            };
            move_cards(&mut machine, 1, source, StackId::D)?;
            if top == next {
                next -= 1;
            } else {
                move_cards(&mut machine, 1, StackId::D, destination)?;
                stats.bypasses += 1;
            }
        }
        std::mem::swap(&mut source, &mut destination);
    }
    Ok(finish(&mut machine, algorithm, stats))
}

fn endpoint_containing(machine: &Machine, card: usize) -> StackId {
    if machine.state().a.contains(&card) {
        StackId::A
    } else {
        debug_assert!(machine.state().b.contains(&card));
        StackId::B
    }
}

fn extract_adaptively(
    machine: &mut Machine,
    mut next: usize,
    stop_after: usize,
    stats: &mut SortStats,
) -> Result<(), MachineError> {
    while next > stop_after {
        let source = endpoint_containing(machine, next);
        let destination = if source == StackId::A {
            StackId::B
        } else {
            StackId::A
        };
        let top = |machine: &Machine| match source {
            StackId::A => machine.state().a[0],
            StackId::B => machine.state().b[0],
            StackId::D => unreachable!(),
        };
        while top(machine) != next {
            move_cards(machine, 1, source, destination)?;
            stats.bypasses += 1;
        }
        move_cards(machine, 1, source, StackId::D)?;
        next -= 1;
    }
    Ok(())
}

fn adaptive_selection(deck: &[usize], algorithm: Algorithm) -> Result<SortResult, MachineError> {
    let mut machine = Machine::new(deck)?;
    let m = active_prefix(deck);
    // Offline knowledge lets setup place cards on opposite endpoints at the
    // maximum's position. The maximum is exposed immediately, so the bypass
    // count is exactly the documented distance between successive maxima.
    let maximum_position = deck[..m]
        .iter()
        .position(|&card| card == m)
        .expect("active prefix contains its maximum");
    for index in 0..m {
        let endpoint = if index <= maximum_position {
            StackId::A
        } else {
            StackId::B
        };
        move_cards(&mut machine, 1, StackId::D, endpoint)?;
    }
    let mut stats = SortStats::default();
    extract_adaptively(&mut machine, m, 0, &mut stats)?;
    Ok(finish(&mut machine, algorithm, stats))
}

fn endpoint_top(machine: &Machine, endpoint: StackId) -> usize {
    match endpoint {
        StackId::A => machine.state().a[0],
        StackId::B => machine.state().b[0],
        StackId::D => unreachable!("D is not an endpoint"),
    }
}

fn extract_with_lookahead(
    machine: &mut Machine,
    mut current: usize,
    stop_after: usize,
    stats: &mut SortStats,
) -> Result<(), MachineError> {
    while current > stop_after {
        let source = endpoint_containing(machine, current);
        let destination = if source == StackId::A {
            StackId::B
        } else {
            StackId::A
        };
        let mut lookahead = current - 1;
        let mut held = 0;

        while endpoint_top(machine, source) != current {
            if lookahead > stop_after && endpoint_top(machine, source) == lookahead {
                move_cards(machine, 1, source, StackId::D)?;
                lookahead -= 1;
                held += 1;
            } else {
                move_cards(machine, 1, source, destination)?;
            }
            stats.bypasses += 1;
        }

        move_cards(machine, held, StackId::D, destination)?;
        move_cards(machine, 1, source, StackId::D)?;
        current -= 1;
    }
    Ok(())
}

const MAX_ROLLOUT_BLOCKERS: usize = 16;

fn blocker_count(machine: &Machine, source: StackId, current: usize) -> usize {
    let stack = match source {
        StackId::A => &machine.state().a,
        StackId::B => &machine.state().b,
        StackId::D => unreachable!("D is not an endpoint"),
    };
    stack
        .iter()
        .position(|&card| card == current)
        .expect("source contains current")
}

fn consecutive_capture_mask(
    machine: &Machine,
    source: StackId,
    current: usize,
    stop_after: usize,
) -> usize {
    let stack = match source {
        StackId::A => &machine.state().a,
        StackId::B => &machine.state().b,
        StackId::D => unreachable!("D is not an endpoint"),
    };
    let mut expected = current - 1;
    let mut mask = 0;
    for (index, &card) in stack
        .iter()
        .take_while(|&&card| card != current)
        .enumerate()
    {
        if expected > stop_after && card == expected {
            mask |= 1 << index;
            expected -= 1;
        }
    }
    mask
}

fn apply_capture_mask(
    machine: &mut Machine,
    current: usize,
    mask: usize,
    stats: &mut SortStats,
) -> Result<(), MachineError> {
    let source = endpoint_containing(machine, current);
    let destination = if source == StackId::A {
        StackId::B
    } else {
        StackId::A
    };
    let mut index = 0;
    let mut held = 0;
    while endpoint_top(machine, source) != current {
        if mask & (1 << index) != 0 {
            move_cards(machine, 1, source, StackId::D)?;
            held += 1;
        } else {
            move_cards(machine, 1, source, destination)?;
        }
        stats.bypasses += 1;
        index += 1;
    }
    move_cards(machine, held, StackId::D, destination)?;
    move_cards(machine, 1, source, StackId::D)
}

/// At each target, tries every way to stage its blockers, scores the resulting
/// state by completing the leaf with ordinary consecutive lookahead, and then
/// commits only the best first pass. Ties retain the ordinary lookahead mask.
fn extract_with_rollout(
    machine: &mut Machine,
    mut current: usize,
    stop_after: usize,
    stats: &mut SortStats,
) -> Result<(), MachineError> {
    while current > stop_after {
        let source = endpoint_containing(machine, current);
        let blockers = blocker_count(machine, source, current);
        if blockers > MAX_ROLLOUT_BLOCKERS {
            return Err(MachineError::InvalidAlgorithmParameter(
                "rollout lookahead supports at most 16 blockers in one pass",
            ));
        }

        let greedy_mask = consecutive_capture_mask(machine, source, current, stop_after);
        let candidate_count = 1usize << blockers;
        let score = |mask| -> Result<usize, MachineError> {
            let mut trial = Machine::from_state(machine.state().clone());
            let mut ignored_stats = SortStats::default();
            apply_capture_mask(&mut trial, current, mask, &mut ignored_stats)?;
            extract_with_lookahead(&mut trial, current - 1, stop_after, &mut ignored_stats)?;
            Ok(trial.plan().len())
        };

        let mut best_mask = greedy_mask;
        let mut best_score = score(greedy_mask)?;
        for mask in 0..candidate_count {
            if mask == greedy_mask {
                continue;
            }
            let candidate_score = score(mask)?;
            if candidate_score < best_score {
                best_score = candidate_score;
                best_mask = mask;
            }
        }

        apply_capture_mask(machine, current, best_mask, stats)?;
        current -= 1;
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct ActiveBucketState {
    a: Vec<usize>,
    b: Vec<usize>,
}

impl ActiveBucketState {
    fn canonical(mut self) -> Self {
        if self.b < self.a {
            std::mem::swap(&mut self.a, &mut self.b);
        }
        self
    }

    fn estimated_owned_vector_bytes(&self) -> usize {
        (self.a.capacity() + self.b.capacity()) * size_of::<usize>()
    }
}

#[derive(Default)]
struct IncrementalRhlPlanner {
    memo: HashMap<ActiveBucketState, usize>,
    algebraic_successors: HashSet<ActiveBucketState>,
    normalized_successors: HashSet<ActiveBucketState>,
    stats: IncrementalRhlStats,
    estimated_owned_vector_bytes: usize,
}

impl IncrementalRhlPlanner {
    fn from_machine(machine: &Machine, low: usize, current: usize) -> ActiveBucketState {
        let project = |stack: &[usize]| {
            stack
                .iter()
                .filter(|&&card| (low..=current).contains(&card))
                .map(|&card| card - low + 1)
                .collect()
        };
        ActiveBucketState {
            a: project(&machine.state().a),
            b: project(&machine.state().b),
        }
    }

    fn source_and_blockers(state: &ActiveBucketState, current: usize) -> (StackId, &[usize]) {
        if let Some(position) = state.a.iter().position(|&card| card == current) {
            (StackId::A, &state.a[..position])
        } else {
            let position = state
                .b
                .iter()
                .position(|&card| card == current)
                .expect("active state contains current target");
            (StackId::B, &state.b[..position])
        }
    }

    fn consecutive_mask(state: &ActiveBucketState, current: usize) -> usize {
        let (_, blockers) = Self::source_and_blockers(state, current);
        let mut expected = current - 1;
        let mut mask = 0;
        for (index, &card) in blockers.iter().enumerate() {
            if expected > 0 && card == expected {
                mask |= 1 << index;
                expected -= 1;
            }
        }
        mask
    }

    fn mask_successor(state: &ActiveBucketState, current: usize, mask: usize) -> ActiveBucketState {
        let (source, blockers) = Self::source_and_blockers(state, current);
        let source_stack = if source == StackId::A {
            &state.a
        } else {
            &state.b
        };
        let current_position = blockers.len();
        let tail = source_stack[current_position + 1..].to_vec();
        let mut staged = Vec::new();
        let mut bypassed = Vec::new();
        for (index, &card) in blockers.iter().enumerate() {
            if mask & (1 << index) != 0 {
                staged.push(card);
            } else {
                bypassed.push(card);
            }
        }
        let old_destination = if source == StackId::A {
            &state.b
        } else {
            &state.a
        };
        let mut destination =
            Vec::with_capacity(staged.len() + bypassed.len() + old_destination.len());
        destination.extend(staged);
        destination.extend(bypassed.into_iter().rev());
        destination.extend_from_slice(old_destination);
        if source == StackId::A {
            ActiveBucketState {
                a: tail,
                b: destination,
            }
        } else {
            ActiveBucketState {
                a: destination,
                b: tail,
            }
        }
    }

    fn remove_forced(
        mut state: ActiveBucketState,
        mut current: usize,
    ) -> (usize, usize, ActiveBucketState) {
        let mut forced = 0;
        while current > 0 {
            if state.a.first() == Some(&current) {
                state.a.remove(0);
            } else if state.b.first() == Some(&current) {
                state.b.remove(0);
            } else {
                break;
            }
            forced += 1;
            current -= 1;
        }
        (forced, current, state)
    }

    fn normalized_successor(state: ActiveBucketState, current: usize) -> ActiveBucketState {
        let (_, _, state) = Self::remove_forced(state, current);
        state.canonical()
    }

    fn update_peak_memory_bytes(&mut self) {
        let memo_table_bytes = self.memo.capacity()
            * (size_of::<ActiveBucketState>() + size_of::<usize>() + size_of::<usize>());
        let successor_table_bytes = (self.algebraic_successors.capacity()
            + self.normalized_successors.capacity())
            * (size_of::<ActiveBucketState>() + size_of::<usize>());
        self.stats.estimated_peak_memory_bytes = self
            .stats
            .estimated_peak_memory_bytes
            .max(self.estimated_owned_vector_bytes + memo_table_bytes + successor_table_bytes);
    }

    fn record_successor(&mut self, successor: &ActiveBucketState, current: usize) {
        if self.algebraic_successors.insert(successor.clone()) {
            self.estimated_owned_vector_bytes += successor.estimated_owned_vector_bytes();
        }
        let normalized = Self::normalized_successor(successor.clone(), current);
        if self.normalized_successors.insert(normalized.clone()) {
            self.estimated_owned_vector_bytes += normalized.estimated_owned_vector_bytes();
        }
        self.update_peak_memory_bytes();
    }

    fn base_cost(&mut self, state: ActiveBucketState, current: usize) -> usize {
        let (forced, current, state) = Self::remove_forced(state, current);
        self.stats.forced_targets_removed += forced;
        if current == 0 {
            return forced;
        }

        let key = state.clone().canonical();
        if let Some(&cost) = self.memo.get(&key) {
            self.stats.base_cache_hits += 1;
            return forced + cost;
        }
        self.stats.base_cache_misses += 1;

        let blockers = Self::source_and_blockers(&state, current).1.len();
        let mask = Self::consecutive_mask(&state, current);
        let successor = Self::mask_successor(&state, current, mask);
        let remainder = 2 * blockers + 1 + self.base_cost(successor, current - 1);
        self.estimated_owned_vector_bytes += key.estimated_owned_vector_bytes();
        self.memo.insert(key, remainder);
        self.stats.base_states_stored = self.memo.len();
        self.update_peak_memory_bytes();
        forced + remainder
    }

    fn best_mask(&mut self, state: &ActiveBucketState, current: usize) -> usize {
        let blockers = Self::source_and_blockers(state, current).1.len();
        let greedy_mask = Self::consecutive_mask(state, current);
        let quotient_count = if blockers == 0 {
            1
        } else {
            1usize << (blockers - 1)
        };
        let pass_cost = 2 * blockers + 1;

        let greedy_successor = Self::mask_successor(state, current, greedy_mask);
        self.stats.masks_visited += 1;
        self.record_successor(&greedy_successor, current - 1);
        let mut best_mask = greedy_mask;
        let mut best_score = pass_cost + self.base_cost(greedy_successor, current - 1);

        for mask in 0..quotient_count {
            if mask == greedy_mask || (blockers > 0 && mask == greedy_mask & (quotient_count - 1)) {
                continue;
            }
            let successor = Self::mask_successor(state, current, mask);
            self.stats.masks_visited += 1;
            self.record_successor(&successor, current - 1);
            let score = pass_cost + self.base_cost(successor, current - 1);
            if score < best_score {
                best_score = score;
                best_mask = mask;
            }
        }
        self.stats.distinct_algebraic_successors = self.algebraic_successors.len();
        self.stats.distinct_normalized_successors = self.normalized_successors.len();
        best_mask
    }
}

#[derive(Default)]
struct TargetBlockDpPlanner {
    memo: HashMap<ActiveBucketState, usize>,
    stats: TargetBlockDpStats,
}

#[derive(Default)]
struct TargetBlockRolloutPlanner {
    suffix: IncrementalRhlPlanner,
    stats: TargetBlockDpStats,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TargetBlockDecision {
    /// Blockers transferred directly before opening the parked block.
    before: usize,
    /// Consecutive source cards parked together on D.
    held: usize,
    /// Cards passed to the destination while the held block remains on D.
    during: usize,
}

impl TargetBlockDpPlanner {
    fn successor(
        state: &ActiveBucketState,
        current: usize,
        decision: TargetBlockDecision,
    ) -> ActiveBucketState {
        let (source, blockers) = IncrementalRhlPlanner::source_and_blockers(state, current);
        let source_stack = if source == StackId::A {
            &state.a
        } else {
            &state.b
        };
        let old_destination = if source == StackId::A {
            &state.b
        } else {
            &state.a
        };
        let after_start = decision.before + decision.held + decision.during;
        debug_assert!(after_start <= blockers.len());
        let before = &blockers[..decision.before];
        let held = &blockers[decision.before..decision.before + decision.held];
        let during = &blockers
            [decision.before + decision.held..decision.before + decision.held + decision.during];
        let after = &blockers[after_start..];
        let mut destination = Vec::with_capacity(blockers.len() + old_destination.len());
        destination.extend(after.iter().rev().copied());
        destination.extend_from_slice(held);
        destination.extend(during.iter().rev().copied());
        destination.extend(before.iter().rev().copied());
        destination.extend_from_slice(old_destination);
        let tail = source_stack[blockers.len() + 1..].to_vec();
        if source == StackId::A {
            ActiveBucketState {
                a: tail,
                b: destination,
            }
        } else {
            ActiveBucketState {
                a: destination,
                b: tail,
            }
        }
    }

    fn descending_target_prefix(stack: &[usize], current: usize) -> usize {
        let mut expected = current.saturating_sub(1);
        let mut length = 0;
        for &card in stack {
            if expected == 0 || card != expected {
                break;
            }
            length += 1;
            expected -= 1;
        }
        length
    }

    /// Enumerates direct finalization plus every single parked reversal block
    /// that exposes a nonempty consecutive-target prefix on the destination.
    fn candidates(
        state: &ActiveBucketState,
        current: usize,
    ) -> Vec<(TargetBlockDecision, ActiveBucketState)> {
        let (source, blockers) = IncrementalRhlPlanner::source_and_blockers(state, current);
        let direct = TargetBlockDecision {
            before: blockers.len(),
            held: 0,
            during: 0,
        };
        let direct_successor = Self::successor(state, current, direct);
        let mut candidates = vec![(direct, direct_successor.clone())];
        let mut seen = HashSet::from([direct_successor]);

        for before in 0..blockers.len() {
            for held in 1..=blockers.len() - before {
                for during in 0..=blockers.len() - before - held {
                    let decision = TargetBlockDecision {
                        before,
                        held,
                        during,
                    };
                    let successor = Self::successor(state, current, decision);
                    let destination = if source == StackId::A {
                        &successor.b
                    } else {
                        &successor.a
                    };
                    if Self::descending_target_prefix(destination, current) == 0 {
                        continue;
                    }
                    if seen.insert(successor.clone()) {
                        candidates.push((decision, successor));
                    }
                }
            }
        }
        candidates
    }

    fn value(&mut self, state: ActiveBucketState, current: usize) -> usize {
        let (forced, current, state) = IncrementalRhlPlanner::remove_forced(state, current);
        self.stats.forced_targets += forced;
        if current == 0 {
            return forced;
        }

        let key = state.clone().canonical();
        if let Some(&cost) = self.memo.get(&key) {
            self.stats.cache_hits += 1;
            return forced + cost;
        }

        let blockers = IncrementalRhlPlanner::source_and_blockers(&state, current)
            .1
            .len();
        let candidates = Self::candidates(&state, current);
        self.stats.max_candidates = self.stats.max_candidates.max(candidates.len());
        self.stats.transitions += candidates.len();
        let pass_cost = 2 * blockers + 1;
        let best = candidates
            .into_iter()
            .map(|(_, successor)| pass_cost + self.value(successor, current - 1))
            .min()
            .expect("direct finalization is always a candidate");
        self.memo.insert(key, best);
        self.stats.states = self.memo.len();
        forced + best
    }

    fn best_decision(&mut self, state: &ActiveBucketState, current: usize) -> TargetBlockDecision {
        let blockers = IncrementalRhlPlanner::source_and_blockers(state, current)
            .1
            .len();
        let candidates = Self::candidates(state, current);
        self.stats.max_candidates = self.stats.max_candidates.max(candidates.len());
        self.stats.transitions += candidates.len();
        let pass_cost = 2 * blockers + 1;
        let mut candidates = candidates.into_iter();
        let (mut best_decision, successor) = candidates.next().expect("direct candidate");
        let mut best = pass_cost + self.value(successor, current - 1);
        for (decision, successor) in candidates {
            let candidate = pass_cost + self.value(successor, current - 1);
            if candidate < best {
                best = candidate;
                best_decision = decision;
            }
        }
        best_decision
    }
}

impl TargetBlockRolloutPlanner {
    fn best_decision(&mut self, state: &ActiveBucketState, current: usize) -> TargetBlockDecision {
        let candidates = TargetBlockDpPlanner::candidates(state, current);
        self.stats.max_candidates = self.stats.max_candidates.max(candidates.len());
        self.stats.transitions += candidates.len();
        let mut candidates = candidates.into_iter();
        let (mut best_decision, successor) = candidates.next().expect("direct candidate");
        let mut best = self.suffix.base_cost(successor, current - 1);
        for (decision, successor) in candidates {
            let candidate = self.suffix.base_cost(successor, current - 1);
            if candidate < best {
                best = candidate;
                best_decision = decision;
            }
        }
        self.stats.states = self.suffix.memo.len();
        self.stats.cache_hits = self.suffix.stats.base_cache_hits;
        self.stats.forced_targets = self.suffix.stats.forced_targets_removed;
        best_decision
    }
}

fn apply_target_block_decision(
    machine: &mut Machine,
    current: usize,
    decision: TargetBlockDecision,
) -> Result<(), MachineError> {
    let source = endpoint_containing(machine, current);
    let destination = opposite_endpoint(source);
    let blockers = blocker_count(machine, source, current);
    let after = blockers - decision.before - decision.held - decision.during;
    move_cards(machine, decision.before, source, destination)?;
    move_cards(machine, decision.held, source, StackId::D)?;
    move_cards(machine, decision.during, source, destination)?;
    move_cards(machine, decision.held, StackId::D, destination)?;
    move_cards(machine, after, source, destination)?;
    move_cards(machine, 1, source, StackId::D)
}

fn extract_with_target_block_dp(
    machine: &mut Machine,
    mut current: usize,
    stop_after: usize,
) -> Result<TargetBlockDpStats, MachineError> {
    let mut planner = TargetBlockDpPlanner::default();
    while current > stop_after {
        let source = endpoint_containing(machine, current);
        if endpoint_top(machine, source) == current {
            move_cards(machine, 1, source, StackId::D)?;
            current -= 1;
            continue;
        }
        let state = IncrementalRhlPlanner::from_machine(machine, stop_after + 1, current);
        let decision = planner.best_decision(&state, current - stop_after);
        apply_target_block_decision(machine, current, decision)?;
        current -= 1;
    }
    Ok(planner.stats)
}

/// Sorts one complete bucket initially occupying A using the globally optimal
/// policy within the consecutive-target block family.
///
/// At target `t`, a candidate either transfers all blockers directly or opens
/// one contiguous parked block on D, passes another source segment above it,
/// and closes the block early. Only outcomes that build a destination prefix
/// `t-1, t-2, ...` are retained. That inverted target block then moves directly
/// onto D as exposed targets.
pub fn target_block_dp_selection_from_a(
    cards: &[usize],
) -> Result<TargetBlockDpResult, MachineError> {
    let state = State::new(cards.to_vec(), Vec::new(), Vec::new())?;
    let mut machine = Machine::from_state(state);
    let stats = extract_with_target_block_dp(&mut machine, cards.len(), 0)?;
    if machine.state() != &State::goal(cards.len()) {
        return Err(MachineError::NotSorted(machine.state().clone()));
    }
    Ok(TargetBlockDpResult {
        plan: machine.take_plan(),
        stats,
    })
}

/// Receding-horizon version of [`target_block_dp_selection_from_a`].
///
/// Every one-block reversal candidate is scored by exact ordinary-lookahead
/// completion, only the first decision is committed, and the suffix-value
/// cache persists across targets. This is the scalable reference policy while
/// the global target-block DP still uses full physical states.
pub fn target_block_rollout_selection_from_a(
    cards: &[usize],
) -> Result<TargetBlockDpResult, MachineError> {
    let state = State::new(cards.to_vec(), Vec::new(), Vec::new())?;
    let mut machine = Machine::from_state(state);
    let stats = extract_with_target_block_rollout(&mut machine, cards.len(), 0)?;
    if machine.state() != &State::goal(cards.len()) {
        return Err(MachineError::NotSorted(machine.state().clone()));
    }
    Ok(TargetBlockDpResult {
        plan: machine.take_plan(),
        stats,
    })
}

fn extract_with_target_block_rollout(
    machine: &mut Machine,
    mut current: usize,
    stop_after: usize,
) -> Result<TargetBlockDpStats, MachineError> {
    let mut planner = TargetBlockRolloutPlanner::default();
    while current > stop_after {
        let source = endpoint_containing(machine, current);
        if endpoint_top(machine, source) == current {
            move_cards(machine, 1, source, StackId::D)?;
            current -= 1;
            continue;
        }
        let state = IncrementalRhlPlanner::from_machine(machine, stop_after + 1, current);
        let decision = planner.best_decision(&state, current - stop_after);
        apply_target_block_decision(machine, current, decision)?;
        current -= 1;
    }
    Ok(planner.stats)
}

fn target_block_rollout_selection(
    deck: &[usize],
    algorithm: Algorithm,
) -> Result<SortResult, MachineError> {
    let mut machine = Machine::new(deck)?;
    let current = active_prefix(deck);
    move_cards(&mut machine, current, StackId::D, StackId::A)?;
    let target_block_dp = extract_with_target_block_rollout(&mut machine, current, 0)?;
    Ok(finish(
        &mut machine,
        algorithm,
        SortStats {
            target_block_dp,
            ..SortStats::default()
        },
    ))
}

fn extract_with_incremental_rhl(
    machine: &mut Machine,
    mut current: usize,
    stop_after: usize,
    stats: &mut SortStats,
) -> Result<(), MachineError> {
    let low = stop_after + 1;
    let mut planner = IncrementalRhlPlanner::default();
    planner.stats.planning_buckets = 1;
    while current > stop_after {
        let source = endpoint_containing(machine, current);
        let blockers = blocker_count(machine, source, current);
        if blockers == 0 {
            move_cards(machine, 1, source, StackId::D)?;
            current -= 1;
            continue;
        }
        if blockers >= usize::BITS as usize {
            return Err(MachineError::InvalidAlgorithmParameter(
                "incremental RHL blocker mask exceeds machine word size",
            ));
        }
        let active_state = IncrementalRhlPlanner::from_machine(machine, low, current);
        let began = Instant::now();
        let best_mask = planner.best_mask(&active_state, current - stop_after);
        planner.stats.planning_nanos += began.elapsed().as_nanos();
        planner.stats.planning_targets += 1;
        apply_capture_mask(machine, current, best_mask, stats)?;
        current -= 1;
    }
    stats.incremental_rhl.masks_visited += planner.stats.masks_visited;
    stats.incremental_rhl.distinct_algebraic_successors +=
        planner.stats.distinct_algebraic_successors;
    stats.incremental_rhl.distinct_normalized_successors +=
        planner.stats.distinct_normalized_successors;
    stats.incremental_rhl.base_cache_hits += planner.stats.base_cache_hits;
    stats.incremental_rhl.base_cache_misses += planner.stats.base_cache_misses;
    stats.incremental_rhl.base_states_stored += planner.stats.base_states_stored;
    stats.incremental_rhl.forced_targets_removed += planner.stats.forced_targets_removed;
    stats.incremental_rhl.estimated_peak_memory_bytes = stats
        .incremental_rhl
        .estimated_peak_memory_bytes
        .max(planner.stats.estimated_peak_memory_bytes);
    stats.incremental_rhl.planning_nanos += planner.stats.planning_nanos;
    stats.incremental_rhl.planning_targets += planner.stats.planning_targets;
    stats.incremental_rhl.planning_buckets += planner.stats.planning_buckets;
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum DepthDecision {
    Stage,
    Bypass,
}

/// One stable-state event in an isolated depth-limited selection trace.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DepthLimitedSelectionTraceEvent {
    /// The initial state, before any selection action.
    Start,
    /// One or more exposed targets were flushed and placed on D.
    PlaceTargets {
        /// Highest target placed by this forced normalization.
        first: usize,
        /// Number of consecutively exposed targets placed.
        count: usize,
    },
    /// A blocker was left temporarily on D.
    Stage {
        /// Card moved from the source endpoint.
        card: usize,
        /// Whether staging was the ordinary consecutive-lookahead choice.
        greedy: bool,
    },
    /// A blocker crossed D to the other endpoint.
    Bypass {
        /// Card moved from the source endpoint.
        card: usize,
        /// Whether bypassing was the ordinary consecutive-lookahead choice.
        greedy: bool,
    },
}

/// One stable state in an isolated depth-limited selection trace.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DepthLimitedSelectionTraceStep {
    /// Event that produced this state.
    pub event: DepthLimitedSelectionTraceEvent,
    /// Primitive moves used by this event.
    pub primitive_cost: usize,
    /// Next target to place, or zero after completing the `1..=n` bucket.
    pub current: usize,
    /// Number of blockers currently staged at the top of D.
    pub held: usize,
    /// Next consecutive card preferred by the greedy policy.
    pub next_capture: usize,
    /// Physical state after the event, with stacks stored top-to-bottom.
    pub state: State,
}

/// Trace of depth-limited selection for one bucket from a recorded configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DepthLimitedSelectionTrace {
    /// Configured blocker-decision horizon.
    pub depth: usize,
    /// Lowest target included in this selection bucket.
    pub low: usize,
    /// Total primitive move count in the isolated selection.
    pub cost: usize,
    /// Initial state and every stable state following an atomic selection event.
    pub steps: Vec<DepthLimitedSelectionTraceStep>,
}

/// Initial physical and partial-pass state for a depth-limited selection trace.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DepthLimitedSelectionTraceConfig {
    /// Arbitrary valid physical configuration, stored top-to-bottom.
    pub state: State,
    /// Lowest target included in the selection bucket.
    pub low: usize,
    /// Next target to place on D.
    pub current: usize,
    /// Number of temporary cards currently staged at the top of D.
    pub held: usize,
    /// Next consecutive card preferred by ordinary lookahead.
    pub next_capture: usize,
}

impl DepthLimitedSelectionTraceConfig {
    /// Infers a full-deck selection state from an arbitrary physical state.
    ///
    /// The maximal goal suffix at the bottom of D is treated as already in
    /// place. Every card above it on D is treated as staged. `next_capture` is
    /// reconstructed by replaying those staged cards in encounter order.
    #[must_use]
    pub fn inferred(state: State) -> Self {
        let n = state.len();
        let mut expected = n;
        let mut in_place = 0;
        for &card in state.d.iter().rev() {
            if card != expected {
                break;
            }
            in_place += 1;
            expected = expected.saturating_sub(1);
        }
        let current = n - in_place;
        let held = state.d.len() - in_place;
        let mut next_capture = current.saturating_sub(1);
        for &card in state.d[..held].iter().rev() {
            if card == next_capture {
                next_capture = next_capture.saturating_sub(1);
            }
        }
        Self {
            state,
            low: 1,
            current,
            held,
            next_capture,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct DepthLimitedState {
    state: State,
    low: usize,
    current: usize,
    source: StackId,
    destination: StackId,
    held: usize,
    next_capture: usize,
}

impl DepthLimitedState {
    fn from_machine(machine: &Machine, low: usize, current: usize) -> Self {
        let source = endpoint_containing(machine, current);
        Self {
            state: machine.state().clone(),
            low,
            current,
            source,
            destination: opposite_endpoint(source),
            held: 0,
            next_capture: current.saturating_sub(1),
        }
    }

    fn estimated_owned_vector_bytes(&self) -> usize {
        (self.state.a.capacity() + self.state.d.capacity() + self.state.b.capacity())
            * size_of::<usize>()
    }

    fn finished(&self) -> bool {
        self.current < self.low
    }

    fn endpoint(&self, id: StackId) -> &[usize] {
        match id {
            StackId::A => &self.state.a,
            StackId::B => &self.state.b,
            StackId::D => unreachable!("D is not an endpoint"),
        }
    }

    fn top_source(&self) -> usize {
        self.endpoint(self.source)[0]
    }

    fn greedy_decision(&self) -> DepthDecision {
        if self.top_source() == self.next_capture {
            DepthDecision::Stage
        } else {
            DepthDecision::Bypass
        }
    }
}

fn opposite_endpoint(endpoint: StackId) -> StackId {
    match endpoint {
        StackId::A => StackId::B,
        StackId::B => StackId::A,
        StackId::D => unreachable!("D is not an endpoint"),
    }
}

fn move_one_in_state(
    state: &mut State,
    source: StackId,
    destination: StackId,
) -> Result<(), MachineError> {
    let movement = match (source, destination) {
        (StackId::A, StackId::D) => Move::AtoD,
        (StackId::D, StackId::A) => Move::DtoA,
        (StackId::D, StackId::B) => Move::DtoB,
        (StackId::B, StackId::D) => Move::BtoD,
        _ => unreachable!("non-primitive move requested"),
    };
    state.apply(movement)
}

fn apply_depth_decision(
    mut state: DepthLimitedState,
    decision: DepthDecision,
) -> Result<(usize, DepthLimitedState), MachineError> {
    let top = state.top_source();
    match decision {
        DepthDecision::Stage => {
            move_one_in_state(&mut state.state, state.source, StackId::D)?;
            state.held += 1;
            if top == state.next_capture {
                state.next_capture = state.next_capture.saturating_sub(1);
            }
            Ok((1, state))
        }
        DepthDecision::Bypass => {
            move_one_in_state(&mut state.state, state.source, StackId::D)?;
            move_one_in_state(&mut state.state, StackId::D, state.destination)?;
            Ok((2, state))
        }
    }
}

fn force_depth_targets(
    mut state: DepthLimitedState,
) -> Result<(usize, DepthLimitedState), MachineError> {
    let mut cost = 0;
    while !state.finished() && state.top_source() == state.current {
        for _ in 0..state.held {
            move_one_in_state(&mut state.state, StackId::D, state.destination)?;
            cost += 1;
        }
        move_one_in_state(&mut state.state, state.source, StackId::D)?;
        cost += 1;
        state.current -= 1;
        state.held = 0;
        state.next_capture = state.current.saturating_sub(1);
        if !state.finished() {
            state.source = if state.state.a.contains(&state.current) {
                StackId::A
            } else {
                debug_assert!(state.state.b.contains(&state.current));
                StackId::B
            };
            state.destination = opposite_endpoint(state.source);
        }
    }
    Ok((cost, state))
}

#[derive(Default)]
struct DepthLimitedRhlPlanner {
    greedy_memo: HashMap<DepthLimitedState, usize>,
    depth_memo: HashMap<(DepthLimitedState, usize), usize>,
    suffix_planner: IncrementalRhlPlanner,
    stats: DepthLimitedRhlStats,
    estimated_owned_vector_bytes: usize,
}

impl DepthLimitedRhlPlanner {
    fn update_peak_memory_bytes(&mut self) {
        let greedy_table_bytes = self.greedy_memo.capacity()
            * (size_of::<DepthLimitedState>() + size_of::<usize>() + size_of::<usize>());
        let depth_table_bytes = self.depth_memo.capacity()
            * (size_of::<(DepthLimitedState, usize)>() + size_of::<usize>() + size_of::<usize>());
        self.stats.estimated_peak_memory_bytes = self.stats.estimated_peak_memory_bytes.max(
            greedy_table_bytes
                + depth_table_bytes
                + self.estimated_owned_vector_bytes
                + self.suffix_planner.stats.estimated_peak_memory_bytes,
        );
    }

    fn active_bucket_state(state: &DepthLimitedState) -> ActiveBucketState {
        let project = |stack: &[usize]| {
            stack
                .iter()
                .filter(|&&card| (state.low..=state.current).contains(&card))
                .map(|&card| card - state.low + 1)
                .collect()
        };
        ActiveBucketState {
            a: project(&state.state.a),
            b: project(&state.state.b),
        }
    }

    fn finish_current_pass_greedily(
        &mut self,
        mut state: DepthLimitedState,
    ) -> Result<(usize, DepthLimitedState), MachineError> {
        let original_current = state.current;
        let mut cost = 0;
        while !state.finished() && state.current == original_current {
            let greedy = state.greedy_decision();
            let (action_cost, child) = apply_depth_decision(state, greedy)?;
            cost += action_cost;
            let (forced_cost, child) = force_depth_targets(child)?;
            cost += forced_cost;
            state = child;
        }
        Ok((cost, state))
    }

    fn greedy_completion_cost(&mut self, state: DepthLimitedState) -> Result<usize, MachineError> {
        self.stats.frontier_evaluations += 1;
        let (forced, state) = force_depth_targets(state)?;
        if state.finished() {
            return Ok(forced);
        }
        if let Some(&cost) = self.greedy_memo.get(&state) {
            self.stats.greedy_cache_hits += 1;
            return Ok(forced + cost);
        }
        self.stats.greedy_cache_misses += 1;
        let (pass_cost, state_after_pass) = self.finish_current_pass_greedily(state.clone())?;
        let suffix_cost = if state_after_pass.finished() {
            0
        } else {
            let suffix_state = Self::active_bucket_state(&state_after_pass);
            self.suffix_planner.base_cost(
                suffix_state,
                state_after_pass.current - state_after_pass.low + 1,
            )
        };
        let cost = pass_cost + suffix_cost;
        self.estimated_owned_vector_bytes += state.estimated_owned_vector_bytes();
        self.greedy_memo.insert(state, cost);
        self.update_peak_memory_bytes();
        Ok(forced + cost)
    }

    fn depth_value(
        &mut self,
        state: DepthLimitedState,
        depth: usize,
    ) -> Result<usize, MachineError> {
        let (forced, state) = force_depth_targets(state)?;
        if state.finished() {
            return Ok(forced);
        }
        if depth == 0 {
            return Ok(forced + self.greedy_completion_cost(state)?);
        }

        let key = (state.clone(), depth);
        if let Some(&cost) = self.depth_memo.get(&key) {
            self.stats.depth_cache_hits += 1;
            return Ok(forced + cost);
        }
        self.stats.depth_cache_misses += 1;
        self.stats.binary_nodes_expanded += 1;

        let greedy = state.greedy_decision();
        let other = match greedy {
            DepthDecision::Stage => DepthDecision::Bypass,
            DepthDecision::Bypass => DepthDecision::Stage,
        };

        let (greedy_cost, greedy_child) = apply_depth_decision(state.clone(), greedy)?;
        let mut best = greedy_cost + self.depth_value(greedy_child, depth - 1)?;
        let (other_cost, other_child) = apply_depth_decision(state.clone(), other)?;
        let candidate = other_cost + self.depth_value(other_child, depth - 1)?;
        if candidate < best {
            best = candidate;
        }

        self.estimated_owned_vector_bytes += key.0.estimated_owned_vector_bytes();
        self.depth_memo.insert(key, best);
        self.update_peak_memory_bytes();
        Ok(forced + best)
    }

    fn best_decision(
        &mut self,
        state: &DepthLimitedState,
        depth: usize,
    ) -> Result<DepthDecision, MachineError> {
        let greedy = state.greedy_decision();
        if depth == 0 {
            return Ok(greedy);
        }
        let other = match greedy {
            DepthDecision::Stage => DepthDecision::Bypass,
            DepthDecision::Bypass => DepthDecision::Stage,
        };

        let before_nodes = self.stats.binary_nodes_expanded;
        let (greedy_cost, greedy_child) = apply_depth_decision(state.clone(), greedy)?;
        let greedy_score = greedy_cost + self.depth_value(greedy_child, depth - 1)?;
        let (other_cost, other_child) = apply_depth_decision(state.clone(), other)?;
        let other_score = other_cost + self.depth_value(other_child, depth - 1)?;
        self.stats.new_nodes_added += self.stats.binary_nodes_expanded - before_nodes;

        if other_score < greedy_score {
            Ok(other)
        } else {
            Ok(greedy)
        }
    }
}

fn execute_depth_forced_targets(
    machine: &mut Machine,
    partial: &mut DepthLimitedState,
) -> Result<(), MachineError> {
    while !partial.finished() && endpoint_top(machine, partial.source) == partial.current {
        move_cards(machine, partial.held, StackId::D, partial.destination)?;
        move_cards(machine, 1, partial.source, StackId::D)?;
        partial.current -= 1;
        partial.held = 0;
        partial.next_capture = partial.current.saturating_sub(1);
        if !partial.finished() {
            partial.source = endpoint_containing(machine, partial.current);
            partial.destination = opposite_endpoint(partial.source);
        }
        partial.state = machine.state().clone();
    }
    Ok(())
}

fn execute_depth_decision(
    machine: &mut Machine,
    partial: &mut DepthLimitedState,
    decision: DepthDecision,
    stats: &mut SortStats,
) -> Result<(), MachineError> {
    let top = endpoint_top(machine, partial.source);
    match decision {
        DepthDecision::Stage => {
            move_cards(machine, 1, partial.source, StackId::D)?;
            partial.held += 1;
            if top == partial.next_capture {
                partial.next_capture = partial.next_capture.saturating_sub(1);
            }
            stats.bypasses += 1;
        }
        DepthDecision::Bypass => {
            move_cards(machine, 1, partial.source, StackId::D)?;
            move_cards(machine, 1, StackId::D, partial.destination)?;
            stats.bypasses += 1;
        }
    }
    partial.state = machine.state().clone();
    Ok(())
}

fn extract_with_depth_limited_rhl(
    machine: &mut Machine,
    current: usize,
    stop_after: usize,
    depth: usize,
    stats: &mut SortStats,
) -> Result<(), MachineError> {
    if depth == 0 {
        extract_with_lookahead(machine, current, stop_after, stats)?;
        stats.depth_limited_rhl.depth = depth;
        stats.depth_limited_rhl.planning_buckets += 1;
        return Ok(());
    }

    let mut partial = DepthLimitedState::from_machine(machine, stop_after + 1, current);
    let mut planner = DepthLimitedRhlPlanner::default();
    planner.stats.depth = depth;
    planner.stats.planning_buckets = 1;
    while !partial.finished() {
        execute_depth_forced_targets(machine, &mut partial)?;
        if partial.finished() {
            break;
        }
        let began = Instant::now();
        let decision = planner.best_decision(&partial, depth)?;
        planner.stats.planning_nanos += began.elapsed().as_nanos();
        planner.stats.planning_decisions += 1;
        execute_depth_decision(machine, &mut partial, decision, stats)?;
    }
    stats.depth_limited_rhl.depth = depth;
    stats.depth_limited_rhl.binary_nodes_expanded += planner.stats.binary_nodes_expanded;
    stats.depth_limited_rhl.frontier_evaluations += planner.stats.frontier_evaluations;
    stats.depth_limited_rhl.greedy_cache_hits += planner.stats.greedy_cache_hits;
    stats.depth_limited_rhl.greedy_cache_misses += planner.stats.greedy_cache_misses;
    stats.depth_limited_rhl.suffix_cache_hits += planner.suffix_planner.stats.base_cache_hits;
    stats.depth_limited_rhl.suffix_cache_misses += planner.suffix_planner.stats.base_cache_misses;
    stats.depth_limited_rhl.suffix_states_stored += planner.suffix_planner.stats.base_states_stored;
    stats.depth_limited_rhl.suffix_forced_targets_removed +=
        planner.suffix_planner.stats.forced_targets_removed;
    stats.depth_limited_rhl.depth_cache_hits += planner.stats.depth_cache_hits;
    stats.depth_limited_rhl.depth_cache_misses += planner.stats.depth_cache_misses;
    stats.depth_limited_rhl.nodes_retained_after_rerooting +=
        planner.stats.nodes_retained_after_rerooting;
    stats.depth_limited_rhl.new_nodes_added += planner.stats.new_nodes_added;
    stats.depth_limited_rhl.estimated_peak_memory_bytes = stats
        .depth_limited_rhl
        .estimated_peak_memory_bytes
        .max(planner.stats.estimated_peak_memory_bytes);
    stats.depth_limited_rhl.planning_nanos += planner.stats.planning_nanos;
    stats.depth_limited_rhl.planning_decisions += planner.stats.planning_decisions;
    stats.depth_limited_rhl.planning_buckets += planner.stats.planning_buckets;
    Ok(())
}

/// Traces depth-limited selection from an arbitrary physical and partial state.
///
/// Trace points are recorded after complete stage, bypass, and forced-placement
/// events; the transient midpoint of the two-move bypass is intentionally
/// omitted. Cards in the fixed portion `state.d[held..]` must fall outside the
/// active target interval `low..=current`.
pub fn trace_depth_limited_selection(
    config: DepthLimitedSelectionTraceConfig,
    depth: usize,
) -> Result<DepthLimitedSelectionTrace, MachineError> {
    config.state.validate()?;
    let n = config.state.len();
    if config.low == 0 || config.low > n.saturating_add(1) {
        return Err(MachineError::InvalidAlgorithmParameter(
            "selection trace has invalid lower target",
        ));
    }
    if config.current > n {
        return Err(MachineError::InvalidAlgorithmParameter(
            "selection trace current target exceeds card count",
        ));
    }
    if config.held > config.state.d.len() {
        return Err(MachineError::InvalidAlgorithmParameter(
            "selection trace held count exceeds D length",
        ));
    }
    if config.current >= config.low && config.next_capture >= config.current {
        return Err(MachineError::InvalidAlgorithmParameter(
            "selection trace next capture must be below current",
        ));
    }
    if config.current < config.low && config.held != 0 {
        return Err(MachineError::InvalidAlgorithmParameter(
            "finished selection trace cannot retain staged cards",
        ));
    }
    if config.state.d[config.held..]
        .iter()
        .any(|&card| (config.low..=config.current).contains(&card))
    {
        return Err(MachineError::InvalidAlgorithmParameter(
            "active target is trapped in the fixed portion of D",
        ));
    }

    let source = if config.current < config.low || config.state.a.contains(&config.current) {
        StackId::A
    } else if config.state.b.contains(&config.current) {
        StackId::B
    } else {
        return Err(MachineError::InvalidAlgorithmParameter(
            "current selection target must be on A or B",
        ));
    };
    let mut partial = DepthLimitedState {
        state: config.state,
        low: config.low,
        current: config.current,
        source,
        destination: opposite_endpoint(source),
        held: config.held,
        next_capture: config.next_capture,
    };
    let mut planner = DepthLimitedRhlPlanner::default();
    let mut cost = 0;
    let mut steps = vec![DepthLimitedSelectionTraceStep {
        event: DepthLimitedSelectionTraceEvent::Start,
        primitive_cost: 0,
        current: partial.current,
        held: partial.held,
        next_capture: partial.next_capture,
        state: partial.state.clone(),
    }];

    while !partial.finished() {
        let first = partial.current;
        let (forced_cost, forced) = force_depth_targets(partial)?;
        partial = forced;
        if partial.current < first {
            cost += forced_cost;
            steps.push(DepthLimitedSelectionTraceStep {
                event: DepthLimitedSelectionTraceEvent::PlaceTargets {
                    first,
                    count: first - partial.current,
                },
                primitive_cost: forced_cost,
                current: partial.current,
                held: partial.held,
                next_capture: partial.next_capture,
                state: partial.state.clone(),
            });
        }
        if partial.finished() {
            break;
        }

        let card = partial.top_source();
        let greedy_decision = partial.greedy_decision();
        let decision = planner.best_decision(&partial, depth)?;
        let (decision_cost, child) = apply_depth_decision(partial, decision)?;
        partial = child;
        cost += decision_cost;
        let greedy = decision == greedy_decision;
        let event = match decision {
            DepthDecision::Stage => DepthLimitedSelectionTraceEvent::Stage { card, greedy },
            DepthDecision::Bypass => DepthLimitedSelectionTraceEvent::Bypass { card, greedy },
        };
        steps.push(DepthLimitedSelectionTraceStep {
            event,
            primitive_cost: decision_cost,
            current: partial.current,
            held: partial.held,
            next_capture: partial.next_capture,
            state: partial.state.clone(),
        });
    }

    Ok(DepthLimitedSelectionTrace {
        depth,
        low: config.low,
        cost,
        steps,
    })
}

/// Traces one complete depth-limited selection bucket initially occupying A.
///
/// `cards` and all recorded stacks use top-to-bottom order.
pub fn trace_depth_limited_selection_from_a(
    cards: &[usize],
    depth: usize,
) -> Result<DepthLimitedSelectionTrace, MachineError> {
    if cards.is_empty() {
        return Err(MachineError::InvalidAlgorithmParameter(
            "selection trace requires at least one card",
        ));
    }
    let state = State::new(cards.to_vec(), Vec::new(), Vec::new())?;
    trace_depth_limited_selection(
        DepthLimitedSelectionTraceConfig {
            state,
            low: 1,
            current: cards.len(),
            held: 0,
            next_capture: cards.len().saturating_sub(1),
        },
        depth,
    )
}

fn project_interval_stack(stack: &[usize], low: usize, high: usize) -> Vec<usize> {
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
}

fn project_interval_state(state: &State, low: usize, high: usize) -> State {
    State {
        a: project_interval_stack(&state.a, low, high),
        d: project_interval_stack(&state.d, low, high),
        b: project_interval_stack(&state.b, low, high),
    }
}

fn interval_heuristic(state: &State, low: usize, high: usize) -> usize {
    transport_heuristic(&project_interval_state(state, low, high))
}

fn stack_top(state: &State, stack: StackId) -> Option<usize> {
    match stack {
        StackId::A => state.a.first().copied(),
        StackId::D => state.d.first().copied(),
        StackId::B => state.b.first().copied(),
    }
}

fn interval_neighbors(state: &State, low: usize, high: usize) -> Vec<(State, Move)> {
    state
        .neighbors()
        .into_iter()
        .filter(|(_, movement)| {
            let (source, _) = movement.endpoints();
            stack_top(state, source).is_some_and(|card| (low..=high).contains(&card))
        })
        .collect()
}

fn extract_with_perfect_selection(
    machine: &mut Machine,
    low: usize,
    high: usize,
) -> Result<(), MachineError> {
    let start = machine.state().clone();
    let goal = State::goal(high - low + 1);
    let start_h = interval_heuristic(&start, low, high);
    let mut open = BinaryHeap::from([Reverse((start_h, 0_usize, start.clone()))]);
    let mut best_g = HashMap::from([(start.clone(), 0_usize)]);
    let mut parents: HashMap<State, (State, Move)> = HashMap::new();
    let mut expanded = HashSet::new();

    while let Some(Reverse((_f, queued_g, state))) = open.pop() {
        if best_g.get(&state) != Some(&queued_g) {
            continue;
        }
        if project_interval_state(&state, low, high) == goal {
            let mut current = state;
            let mut reverse_plan = Vec::with_capacity(queued_g);
            while current != start {
                let (parent, movement) = parents[&current].clone();
                reverse_plan.push(movement);
                current = parent;
            }
            reverse_plan.reverse();
            return machine.apply_plan(&reverse_plan);
        }

        expanded.insert(state.clone());
        for (child, movement) in interval_neighbors(&state, low, high) {
            let candidate_g = queued_g + 1;
            if best_g.get(&child).is_none_or(|&known| candidate_g < known) {
                best_g.insert(child.clone(), candidate_g);
                parents.insert(child.clone(), (state.clone(), movement));
                if expanded.contains(&child) {
                    expanded.remove(&child);
                }
                let f = candidate_g + interval_heuristic(&child, low, high);
                open.push(Reverse((f, candidate_g, child)));
            }
        }
    }
    unreachable!("the current interval can always be extracted to D")
}

fn lookahead_selection(deck: &[usize], algorithm: Algorithm) -> Result<SortResult, MachineError> {
    let mut machine = Machine::new(deck)?;
    let m = active_prefix(deck);
    move_cards(&mut machine, m, StackId::D, StackId::A)?;

    let mut stats = SortStats::default();
    extract_with_lookahead(&mut machine, m, 0, &mut stats)?;

    Ok(finish(&mut machine, algorithm, stats))
}

fn repartition_endpoint(
    machine: &mut Machine,
    count: usize,
    source: StackId,
    split: usize,
) -> Result<(), MachineError> {
    move_cards(machine, count, source, StackId::D)?;
    for _ in 0..count {
        let destination = if machine.state().d[0] <= split {
            StackId::A
        } else {
            StackId::B
        };
        move_cards(machine, 1, StackId::D, destination)?;
    }
    Ok(())
}

fn lower_partition_size(card_count: usize, bucket_count: usize, lower_buckets: usize) -> usize {
    let minimum_bucket_size = card_count / bucket_count;
    let larger_buckets = card_count % bucket_count;
    lower_buckets * minimum_bucket_size + larger_buckets.min(lower_buckets)
}

fn extract_partition_tree(
    machine: &mut Machine,
    low: usize,
    high: usize,
    bucket_count: usize,
    source: StackId,
    leaf_selection: LeafSelection,
    stats: &mut SortStats,
) -> Result<(), MachineError> {
    let card_count = high - low + 1;
    if bucket_count == 1 {
        return match leaf_selection {
            LeafSelection::Consecutive => extract_with_lookahead(machine, high, low - 1, stats),
            LeafSelection::Rollout => extract_with_rollout(machine, high, low - 1, stats),
            LeafSelection::IncrementalRhl => {
                extract_with_incremental_rhl(machine, high, low - 1, stats)
            }
            LeafSelection::DepthLimitedRhl { depth } => {
                extract_with_depth_limited_rhl(machine, high, low - 1, depth, stats)
            }
            LeafSelection::TargetBlock => {
                let leaf = extract_with_target_block_rollout(machine, high, low - 1)?;
                stats.target_block_dp.states += leaf.states;
                stats.target_block_dp.transitions += leaf.transitions;
                stats.target_block_dp.max_candidates = stats
                    .target_block_dp
                    .max_candidates
                    .max(leaf.max_candidates);
                stats.target_block_dp.cache_hits += leaf.cache_hits;
                stats.target_block_dp.forced_targets += leaf.forced_targets;
                Ok(())
            }
            LeafSelection::Perfect => extract_with_perfect_selection(machine, low, high),
        };
    }

    let lower_buckets = bucket_count / 2;
    let upper_buckets = bucket_count - lower_buckets;
    let lower_cards = lower_partition_size(card_count, bucket_count, lower_buckets);
    let split = low + lower_cards - 1;

    repartition_endpoint(machine, card_count, source, split)?;
    extract_partition_tree(
        machine,
        split + 1,
        high,
        upper_buckets,
        StackId::B,
        leaf_selection,
        stats,
    )?;
    extract_partition_tree(
        machine,
        low,
        split,
        lower_buckets,
        StackId::A,
        leaf_selection,
        stats,
    )
}

#[derive(Clone, Copy)]
enum LeafSelection {
    Consecutive,
    Rollout,
    IncrementalRhl,
    DepthLimitedRhl { depth: usize },
    TargetBlock,
    Perfect,
}

fn partition_lookahead_selection(
    deck: &[usize],
    algorithm: Algorithm,
    requested_buckets: usize,
    leaf_selection: LeafSelection,
) -> Result<SortResult, MachineError> {
    let mut machine = Machine::new(deck)?;
    let m = active_prefix(deck);
    let bucket_count = requested_buckets.min(m);
    debug_assert!(
        bucket_count >= 2,
        "non-sorted active prefixes have at least two cards"
    );
    let largest_bucket = m.div_ceil(bucket_count);
    if matches!(leaf_selection, LeafSelection::Rollout) && largest_bucket > MAX_ROLLOUT_BLOCKERS + 1
    {
        return Err(MachineError::InvalidAlgorithmParameter(
            "rollout lookahead requires value buckets of at most 17 cards",
        ));
    }
    let lower_buckets = bucket_count / 2;
    let upper_buckets = bucket_count - lower_buckets;
    let lower_cards = lower_partition_size(m, bucket_count, lower_buckets);

    // The root partition starts on D, so it costs one move per card rather
    // than the two moves needed to repartition an endpoint below.
    for _ in 0..m {
        let destination = if machine.state().d[0] <= lower_cards {
            StackId::A
        } else {
            StackId::B
        };
        move_cards(&mut machine, 1, StackId::D, destination)?;
    }

    let mut stats = SortStats::default();
    extract_partition_tree(
        &mut machine,
        lower_cards + 1,
        m,
        upper_buckets,
        StackId::B,
        leaf_selection,
        &mut stats,
    )?;
    extract_partition_tree(
        &mut machine,
        1,
        lower_cards,
        lower_buckets,
        StackId::A,
        leaf_selection,
        &mut stats,
    )?;

    Ok(finish(&mut machine, algorithm, stats))
}

fn binary_presort(deck: &[usize], algorithm: Algorithm) -> Result<SortResult, MachineError> {
    let mut machine = Machine::new(deck)?;
    let split = deck.len() / 2;
    for _ in 0..deck.len() {
        let destination = if machine.state().d[0] <= split {
            StackId::A
        } else {
            StackId::B
        };
        move_cards(&mut machine, 1, StackId::D, destination)?;
    }
    let mut stats = SortStats::default();
    extract_adaptively(&mut machine, deck.len(), split, &mut stats)?;
    extract_adaptively(&mut machine, split, 0, &mut stats)?;
    Ok(finish(&mut machine, algorithm, stats))
}

fn merge_segment(machine: &mut Machine, k: usize) -> Result<(), MachineError> {
    if k <= 1 {
        return Ok(());
    }
    let a = k.div_ceil(2);
    let b = k / 2;
    move_cards(machine, a, StackId::D, StackId::A)?;
    move_cards(machine, b, StackId::D, StackId::B)?;
    move_cards(machine, a, StackId::A, StackId::D)?;
    merge_segment(machine, a)?;
    move_cards(machine, a, StackId::D, StackId::A)?;
    move_cards(machine, b, StackId::B, StackId::D)?;
    merge_segment(machine, b)?;
    move_cards(machine, b, StackId::D, StackId::B)?;
    let mut a_left = a;
    let mut b_left = b;
    while a_left > 0 && b_left > 0 {
        if machine.state().a[0] >= machine.state().b[0] {
            machine.apply(Move::AtoD)?;
            a_left -= 1;
        } else {
            machine.apply(Move::BtoD)?;
            b_left -= 1;
        }
    }
    move_cards(machine, a_left, StackId::A, StackId::D)?;
    move_cards(machine, b_left, StackId::B, StackId::D)?;
    Ok(())
}

fn merge_sort(deck: &[usize], algorithm: Algorithm) -> Result<SortResult, MachineError> {
    let mut machine = Machine::new(deck)?;
    merge_segment(&mut machine, deck.len())?;
    Ok(finish(&mut machine, algorithm, SortStats::default()))
}

fn msb_segment(machine: &mut Machine, low: usize, high: usize) -> Result<(), MachineError> {
    let k = high.saturating_sub(low) + 1;
    if k <= 1 {
        return Ok(());
    }
    let a = k / 2;
    let split = low + a - 1;
    for _ in 0..k {
        let destination = if machine.state().d[0] <= split {
            StackId::A
        } else {
            StackId::B
        };
        move_cards(machine, 1, StackId::D, destination)?;
    }
    move_cards(machine, a, StackId::A, StackId::D)?;
    msb_segment(machine, low, split)?;
    move_cards(machine, a, StackId::D, StackId::A)?;
    let b = k - a;
    move_cards(machine, b, StackId::B, StackId::D)?;
    msb_segment(machine, split + 1, high)?;
    move_cards(machine, a, StackId::A, StackId::D)
}

fn msb_radix(deck: &[usize], algorithm: Algorithm) -> Result<SortResult, MachineError> {
    let mut machine = Machine::new(deck)?;
    if !deck.is_empty() {
        msb_segment(&mut machine, 1, deck.len())?;
    }
    Ok(finish(&mut machine, algorithm, SortStats::default()))
}

fn lsb_radix(deck: &[usize], algorithm: Algorithm) -> Result<SortResult, MachineError> {
    let mut machine = Machine::new(deck)?;
    let bits = if deck.len() <= 1 {
        0
    } else {
        usize::BITS as usize - (deck.len() - 1).leading_zeros() as usize
    };
    for bit in 0..bits {
        for _ in 0..deck.len() {
            let destination = if ((machine.state().d[0] - 1) >> bit) & 1 == 0 {
                StackId::A
            } else {
                StackId::B
            };
            move_cards(&mut machine, 1, StackId::D, destination)?;
        }
        let b = machine.state().b.len();
        let a = machine.state().a.len();
        move_cards(&mut machine, b, StackId::B, StackId::D)?;
        move_cards(&mut machine, a, StackId::A, StackId::D)?;
    }
    let stats = SortStats {
        phases: bits,
        ..SortStats::default()
    };
    Ok(finish(&mut machine, algorithm, stats))
}

fn ascending_runs(values: &[usize]) -> Vec<usize> {
    if values.is_empty() {
        return Vec::new();
    }
    let mut runs = Vec::new();
    let mut length = 1;
    for pair in values.windows(2) {
        if pair[0] < pair[1] {
            length += 1;
        } else {
            runs.push(length);
            length = 1;
        }
    }
    runs.push(length);
    runs
}

fn merge_endpoint_sequences(
    machine: &mut Machine,
    mut a_segments: Vec<usize>,
    mut b_segments: Vec<usize>,
) -> Result<Vec<usize>, MachineError> {
    let mut completed_reversed = Vec::new();
    while !a_segments.is_empty() && !b_segments.is_empty() {
        let a_len = a_segments.pop().expect("checked nonempty");
        let b_len = b_segments.pop().expect("checked nonempty");
        let mut a_left = a_len;
        let mut b_left = b_len;
        while a_left > 0 && b_left > 0 {
            if machine.state().a[0] >= machine.state().b[0] {
                machine.apply(Move::AtoD)?;
                a_left -= 1;
            } else {
                machine.apply(Move::BtoD)?;
                b_left -= 1;
            }
        }
        move_cards(machine, a_left, StackId::A, StackId::D)?;
        move_cards(machine, b_left, StackId::B, StackId::D)?;
        completed_reversed.push(a_len + b_len);
    }
    if let Some(length) = a_segments.pop() {
        debug_assert!(a_segments.is_empty() && b_segments.is_empty());
        move_cards(machine, length, StackId::A, StackId::D)?;
        completed_reversed.push(length);
    } else if let Some(length) = b_segments.pop() {
        debug_assert!(a_segments.is_empty() && b_segments.is_empty());
        move_cards(machine, length, StackId::B, StackId::D)?;
        completed_reversed.push(length);
    }
    completed_reversed.reverse();
    Ok(completed_reversed)
}

fn natural_pass(machine: &mut Machine, runs: &[usize]) -> Result<Vec<usize>, MachineError> {
    let mut a_segments = Vec::new();
    let mut b_segments = Vec::new();
    for (index, &length) in runs.iter().enumerate() {
        let (endpoint, segments) = if index % 2 == 0 {
            (StackId::A, &mut a_segments)
        } else {
            (StackId::B, &mut b_segments)
        };
        move_cards(machine, length, StackId::D, endpoint)?;
        segments.push(length);
    }
    merge_endpoint_sequences(machine, a_segments, b_segments)
}

fn natural_sort(deck: &[usize], algorithm: Algorithm) -> Result<SortResult, MachineError> {
    let mut machine = Machine::new(deck)?;
    let mut runs = ascending_runs(deck);
    let initial_runs = runs.len();
    let mut phases = 0;
    while runs.len() > 1 {
        runs = natural_pass(&mut machine, &runs)?;
        phases += 1;
    }
    let stats = SortStats {
        phases,
        initial_runs,
        ..SortStats::default()
    };
    Ok(finish(&mut machine, algorithm, stats))
}

/// An alphabetic merge tree whose leaves are maximal ascending runs in their
/// original top-to-bottom order.
#[derive(Clone, Debug, Eq, PartialEq)]
enum AlphabeticMergeTree {
    Leaf(usize),
    Node {
        upper: Box<Self>,
        lower: Box<Self>,
        size: usize,
    },
}

impl AlphabeticMergeTree {
    fn size(&self) -> usize {
        match self {
            Self::Leaf(size) | Self::Node { size, .. } => *size,
        }
    }
}

/// Builds a minimum-weight alphabetic binary tree by the interval recurrence
/// `C(i,j) = sum(i..j) + min_k(C(i,k) + C(k+1,j))`.
fn optimal_alphabetic_tree(run_sizes: &[usize]) -> AlphabeticMergeTree {
    debug_assert!(!run_sizes.is_empty());
    let run_count = run_sizes.len();
    let mut prefix = vec![0; run_count + 1];
    for (index, &size) in run_sizes.iter().enumerate() {
        prefix[index + 1] = prefix[index] + size;
    }

    let mut costs = vec![vec![0; run_count]; run_count];
    let mut splits = vec![vec![0; run_count]; run_count];
    for length in 2..=run_count {
        for first in 0..=run_count - length {
            let last = first + length - 1;
            let interval_size = prefix[last + 1] - prefix[first];
            let (split, subcost) = (first..last)
                .map(|split| (split, costs[first][split] + costs[split + 1][last]))
                .min_by_key(|&(split, cost)| (cost, split))
                .expect("nontrivial interval has a split");
            costs[first][last] = interval_size + subcost;
            splits[first][last] = split;
        }
    }

    fn build(
        first: usize,
        last: usize,
        run_sizes: &[usize],
        prefix: &[usize],
        splits: &[Vec<usize>],
    ) -> AlphabeticMergeTree {
        if first == last {
            return AlphabeticMergeTree::Leaf(run_sizes[first]);
        }
        let split = splits[first][last];
        AlphabeticMergeTree::Node {
            upper: Box::new(build(first, split, run_sizes, prefix, splits)),
            lower: Box::new(build(split + 1, last, run_sizes, prefix, splits)),
            size: prefix[last + 1] - prefix[first],
        }
    }

    build(0, run_count - 1, run_sizes, &prefix, &splits)
}

fn realize_alphabetic_tree(
    machine: &mut Machine,
    tree: &AlphabeticMergeTree,
) -> Result<(), MachineError> {
    let AlphabeticMergeTree::Node { upper, lower, .. } = tree else {
        return Ok(());
    };

    // The upper child is exposed first. Once sorted, parking it on A exposes
    // the lower child. Moving both ascending children to endpoints exposes
    // their maxima, so taking the larger maximum first builds one ascending
    // run on D.
    realize_alphabetic_tree(machine, upper)?;
    move_cards(machine, upper.size(), StackId::D, StackId::A)?;
    realize_alphabetic_tree(machine, lower)?;
    move_cards(machine, lower.size(), StackId::D, StackId::B)?;

    let mut a_left = upper.size();
    let mut b_left = lower.size();
    while a_left > 0 && b_left > 0 {
        if machine.state().a[0] >= machine.state().b[0] {
            machine.apply(Move::AtoD)?;
            a_left -= 1;
        } else {
            machine.apply(Move::BtoD)?;
            b_left -= 1;
        }
    }
    move_cards(machine, a_left, StackId::A, StackId::D)?;
    move_cards(machine, b_left, StackId::B, StackId::D)
}

fn hu_tucker_natural_merge(
    deck: &[usize],
    algorithm: Algorithm,
) -> Result<SortResult, MachineError> {
    let runs = ascending_runs(deck);
    let initial_runs = runs.len();
    let tree = optimal_alphabetic_tree(&runs);
    let mut machine = Machine::new(deck)?;
    realize_alphabetic_tree(&mut machine, &tree)?;
    Ok(finish(
        &mut machine,
        algorithm,
        SortStats {
            phases: initial_runs.saturating_sub(1),
            initial_runs,
            ..SortStats::default()
        },
    ))
}

fn is_strictly_descending(deck: &[usize]) -> bool {
    deck.windows(2).all(|pair| pair[0] > pair[1])
}

fn signed_natural(deck: &[usize], algorithm: Algorithm) -> Result<SortResult, MachineError> {
    if is_strictly_descending(deck) {
        let mut machine = Machine::new(deck)?;
        reverse_d(&mut machine)?;
        return Ok(finish(
            &mut machine,
            algorithm,
            SortStats {
                reversals: 1,
                initial_runs: deck.len(),
                ..SortStats::default()
            },
        ));
    }
    // The document explicitly permits an ordinary-natural fallback while the
    // pure signed phase rule remains unresolved.
    let mut result = natural_sort(deck, algorithm)?;
    result.algorithm = algorithm;
    Ok(result)
}

fn one_increasing(values: &[usize]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

/// Returns a deterministic two-coloring into increasing subsequences.
fn two_increasing_coloring(values: &[usize]) -> Option<Vec<bool>> {
    // `(tail_a, tail_b)` maps to the lexicographically smallest membership
    // vector. `false` (A) sorts before `true` (B).
    let mut states = BTreeMap::from([((0, 0), Vec::new())]);
    for &value in values {
        let mut next: BTreeMap<(usize, usize), Vec<bool>> = BTreeMap::new();
        for (&(tail_a, tail_b), bits) in &states {
            if value > tail_a {
                let mut candidate = bits.clone();
                candidate.push(false);
                update_coloring(&mut next, (value, tail_b), candidate);
            }
            if value > tail_b {
                let mut candidate = bits.clone();
                candidate.push(true);
                update_coloring(&mut next, (tail_a, value), candidate);
            }
        }
        states = next;
        if states.is_empty() {
            return None;
        }
    }
    states.into_values().min()
}

fn update_coloring(
    states: &mut BTreeMap<(usize, usize), Vec<bool>>,
    tails: (usize, usize),
    candidate: Vec<bool>,
) {
    match states.get_mut(&tails) {
        Some(current) if candidate < *current => *current = candidate,
        None => {
            states.insert(tails, candidate);
        }
        Some(_) => {}
    }
}

fn split_merge_phase(machine: &mut Machine, blocks: &[usize]) -> Result<Vec<usize>, MachineError> {
    let mut remaining_blocks = blocks;
    let mut a_segments = Vec::new();
    let mut b_segments = Vec::new();
    while !remaining_blocks.is_empty() {
        let mut length = 0;
        let mut chosen = None;
        let mut consumed_blocks = 0;
        for (index, &block_len) in remaining_blocks.iter().enumerate() {
            length += block_len;
            let prefix = &machine.state().d[..length];
            let coloring = if one_increasing(prefix) {
                Some(vec![false; length])
            } else {
                two_increasing_coloring(prefix)
            };
            if let Some(bits) = coloring {
                chosen = Some(bits);
                consumed_blocks = index + 1;
            }
        }
        let bits = chosen.expect("one complete ascending block is feasible");
        let one_output = bits.iter().all(|&bit| bit == bits[0]);
        if one_output {
            let endpoint = if a_segments.len() <= b_segments.len() {
                StackId::A
            } else {
                StackId::B
            };
            let len = bits.len();
            move_cards(machine, len, StackId::D, endpoint)?;
            if endpoint == StackId::A {
                a_segments.push(len);
            } else {
                b_segments.push(len);
            }
        } else {
            let mut a_len = 0;
            let mut b_len = 0;
            for bit in bits {
                if bit {
                    machine.apply(Move::DtoB)?;
                    b_len += 1;
                } else {
                    machine.apply(Move::DtoA)?;
                    a_len += 1;
                }
            }
            debug_assert!(a_len > 0 && b_len > 0);
            a_segments.push(a_len);
            b_segments.push(b_len);
        }
        remaining_blocks = &remaining_blocks[consumed_blocks..];
    }
    merge_endpoint_sequences(machine, a_segments, b_segments)
}

fn split_merge(deck: &[usize], algorithm: Algorithm) -> Result<SortResult, MachineError> {
    let mut machine = Machine::new(deck)?;
    let initial_runs = ascending_runs(deck).len();
    let mut phases = 0;
    while !one_increasing(&machine.state().d) {
        let blocks = ascending_runs(&machine.state().d);
        split_merge_phase(&mut machine, &blocks)?;
        phases += 1;
    }
    let stats = SortStats {
        phases,
        initial_runs,
        ..SortStats::default()
    };
    Ok(finish(&mut machine, algorithm, stats))
}

fn reversing_split_merge(deck: &[usize], algorithm: Algorithm) -> Result<SortResult, MachineError> {
    if is_strictly_descending(deck) {
        let mut machine = Machine::new(deck)?;
        reverse_d_to_endpoint(&mut machine, deck.len(), StackId::A)?;
        move_cards(&mut machine, deck.len(), StackId::A, StackId::D)?;
        return Ok(finish(
            &mut machine,
            algorithm,
            SortStats {
                phases: 1,
                reversals: 1,
                initial_runs: deck.len(),
                ..SortStats::default()
            },
        ));
    }
    // Safe experimental mode retains split-merge's certified bound.
    split_merge(deck, algorithm)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::Rng;
    use crate::validate_sort_plan;

    fn permutations(values: &mut [usize], start: usize, visit: &mut impl FnMut(&[usize])) {
        if start == values.len() {
            visit(values);
            return;
        }
        for index in start..values.len() {
            values.swap(start, index);
            permutations(values, start + 1, visit);
            values.swap(start, index);
        }
    }

    #[test]
    fn all_algorithms_sort_every_permutation_through_seven() {
        for n in 0..=7 {
            let mut deck: Vec<_> = (1..=n).collect();
            permutations(&mut deck, 0, &mut |permutation| {
                for algorithm in Algorithm::ALL {
                    let result = solve(algorithm, permutation).unwrap_or_else(|error| {
                        panic!("{} failed on {permutation:?}: {error}", algorithm.name())
                    });
                    validate_sort_plan(permutation, &result.plan).unwrap_or_else(|error| {
                        panic!("{} invalid on {permutation:?}: {error}", algorithm.name())
                    });
                    if algorithm == Algorithm::LookaheadSelection {
                        let m = active_prefix(permutation);
                        assert_eq!(result.cost(), 2 * m + 2 * result.stats.bypasses);
                        assert!(result.cost() <= m * m + m);
                    }
                    if algorithm == Algorithm::TwoKPartitionLookaheadSelection(1) {
                        let m = active_prefix(permutation);
                        let a = m / 2;
                        let b = m - a;
                        assert_eq!(result.cost(), 2 * m + 2 * result.stats.bypasses);
                        assert!(
                            result.cost()
                                <= 2 * m + a * a.saturating_sub(1) + b * b.saturating_sub(1)
                        );
                    }
                }
            });
        }
    }

    #[test]
    fn lookahead_selection_freezes_suffix_and_accounts_for_staging() {
        let sorted = solve(Algorithm::LookaheadSelection, &[1, 2, 3, 4]).unwrap();
        assert!(sorted.plan.is_empty());
        assert_eq!(sorted.stats.bypasses, 0);

        let result = solve(Algorithm::LookaheadSelection, &[2, 1, 3, 4]).unwrap();
        validate_sort_plan(&[2, 1, 3, 4], &result.plan).unwrap();
        assert_eq!(result.stats.bypasses, 1);
        assert_eq!(result.cost(), 6);

        let active = active_prefix(&[2, 1, 3, 4]);
        assert_eq!(result.cost(), 2 * active + 2 * result.stats.bypasses);
    }

    #[test]
    fn two_k_partition_lookahead_freezes_suffix_and_keeps_buckets_separate() {
        let algorithm = Algorithm::TwoKPartitionLookaheadSelection(1);
        let sorted = solve(algorithm, &[1, 2, 3, 4]).unwrap();
        assert!(sorted.plan.is_empty());

        let result = solve(algorithm, &[2, 1, 3, 4]).unwrap();
        validate_sort_plan(&[2, 1, 3, 4], &result.plan).unwrap();
        assert_eq!(result.stats.bypasses, 0);
        assert_eq!(result.cost(), 4);
    }

    #[test]
    fn two_k_partition_name_round_trips() {
        let algorithm = Algorithm::TwoKPartitionLookaheadSelection(17);
        assert_eq!(Algorithm::from_name(&algorithm.name()), Some(algorithm));
        assert_eq!(
            Algorithm::from_name("2k-partition-lookahead-selection:0"),
            None
        );

        let rollout = Algorithm::RolloutTwoKPartitionLookaheadSelectionExperimental(2);
        assert_eq!(Algorithm::from_name(&rollout.name()), Some(rollout));

        let incremental = Algorithm::IncrementalRhlTwoKPartitionLookaheadSelectionExperimental(2);
        assert_eq!(Algorithm::from_name(&incremental.name()), Some(incremental));

        let depth_limited =
            Algorithm::DepthLimitedRhlTwoKPartitionLookaheadSelectionExperimental(2, 13);
        assert_eq!(
            Algorithm::from_name(&depth_limited.name()),
            Some(depth_limited)
        );
        assert_eq!(
            Algorithm::from_name(
                "depth-limited-rhl-2k-partition-lookahead-selection-experimental:0:2"
            ),
            None
        );

        let perfect = Algorithm::TwoKPartitioningPerfectSelection(2);
        assert_eq!(Algorithm::from_name(&perfect.name()), Some(perfect));
        assert_eq!(
            Algorithm::from_name("2k-partitioning-perfect-selection:0"),
            None
        );

        let target_block = Algorithm::TargetBlockRolloutSelectionExperimental;
        assert_eq!(
            Algorithm::from_name(&target_block.name()),
            Some(target_block)
        );
        let partitioned_target_block =
            Algorithm::TargetBlockRolloutTwoKPartitionSelectionExperimental(2);
        assert_eq!(
            Algorithm::from_name(&partitioned_target_block.name()),
            Some(partitioned_target_block)
        );
        assert_eq!(
            Algorithm::from_name("target-block-rollout-2k-partition-selection-experimental:0"),
            None
        );
    }

    #[test]
    fn rollout_lookahead_sorts_and_never_loses_to_consecutive_lookahead() {
        let mut found_strict_improvement = false;
        for n in 2..=8 {
            let mut deck: Vec<_> = (1..=n).collect();
            permutations(&mut deck, 0, &mut |permutation| {
                let consecutive =
                    solve(Algorithm::TwoKPartitionLookaheadSelection(1), permutation).unwrap();
                let rollout = solve(
                    Algorithm::RolloutTwoKPartitionLookaheadSelectionExperimental(1),
                    permutation,
                )
                .unwrap();
                validate_sort_plan(permutation, &rollout.plan).unwrap();
                assert!(
                    rollout.cost() <= consecutive.cost(),
                    "rollout cost {} exceeded consecutive cost {} on {permutation:?}",
                    rollout.cost(),
                    consecutive.cost()
                );
                found_strict_improvement |= rollout.cost() < consecutive.cost();
            });
        }
        assert!(found_strict_improvement);
    }

    #[test]
    fn incremental_rhl_matches_brute_force_scores_and_masks_on_small_leaves() {
        for n in 1..=6 {
            let mut cards: Vec<_> = (1..=n).collect();
            permutations(&mut cards, 0, &mut |permutation| {
                for cut in 0..=n {
                    let state = ActiveBucketState {
                        a: permutation[..cut].to_vec(),
                        b: permutation[cut..].to_vec(),
                    };
                    let (source, blockers) = IncrementalRhlPlanner::source_and_blockers(&state, n);
                    let greedy_mask = IncrementalRhlPlanner::consecutive_mask(&state, n);
                    let mut brute_mask = greedy_mask;
                    let score = |mask| {
                        let machine_state = State {
                            a: state.a.clone(),
                            d: Vec::new(),
                            b: state.b.clone(),
                        };
                        let mut machine = Machine::from_state(machine_state);
                        let mut ignored = SortStats::default();
                        apply_capture_mask(&mut machine, n, mask, &mut ignored).unwrap();
                        extract_with_lookahead(&mut machine, n - 1, 0, &mut ignored).unwrap();
                        machine.plan().len()
                    };
                    let mut brute_score = score(greedy_mask);
                    for mask in 0..(1usize << blockers.len()) {
                        if mask == greedy_mask {
                            continue;
                        }
                        let candidate_score = score(mask);
                        if candidate_score < brute_score {
                            brute_score = candidate_score;
                            brute_mask = mask;
                        }
                    }

                    let mut planner = IncrementalRhlPlanner::default();
                    let incremental_mask = planner.best_mask(&state, n);
                    let successor =
                        IncrementalRhlPlanner::mask_successor(&state, n, incremental_mask);
                    let incremental_score =
                        2 * blockers.len() + 1 + planner.base_cost(successor, n - 1);
                    assert_eq!(
                        (incremental_score, incremental_mask),
                        (brute_score, brute_mask),
                        "mismatch for n={n}, source={source:?}, state={state:?}"
                    );
                }
            });
        }
    }

    #[test]
    fn incremental_rhl_emits_the_same_plans_as_rollout() {
        let mut rng = Rng::new(0x1c3_3e7a);
        for k in 2..=4 {
            for _ in 0..100 {
                let deck = rng.permutation(24);
                let brute = solve(
                    Algorithm::RolloutTwoKPartitionLookaheadSelectionExperimental(k),
                    &deck,
                )
                .unwrap();
                let incremental = solve(
                    Algorithm::IncrementalRhlTwoKPartitionLookaheadSelectionExperimental(k),
                    &deck,
                )
                .unwrap();
                assert_eq!(
                    incremental.plan, brute.plan,
                    "plan mismatch for k={k}, deck={deck:?}"
                );
                validate_sort_plan(&deck, &incremental.plan).unwrap();
            }
        }
    }

    fn reference_depth_value(
        state: DepthLimitedState,
        depth: usize,
    ) -> Result<usize, MachineError> {
        let (forced, state) = force_depth_targets(state)?;
        if state.finished() {
            return Ok(forced);
        }
        if depth == 0 {
            let mut planner = DepthLimitedRhlPlanner::default();
            return Ok(forced + planner.greedy_completion_cost(state)?);
        }
        let greedy = state.greedy_decision();
        let other = match greedy {
            DepthDecision::Stage => DepthDecision::Bypass,
            DepthDecision::Bypass => DepthDecision::Stage,
        };
        let (greedy_cost, greedy_child) = apply_depth_decision(state.clone(), greedy)?;
        let greedy_score = greedy_cost + reference_depth_value(greedy_child, depth - 1)?;
        let (other_cost, other_child) = apply_depth_decision(state, other)?;
        let other_score = other_cost + reference_depth_value(other_child, depth - 1)?;
        Ok(forced + greedy_score.min(other_score))
    }

    #[test]
    fn depth_limited_rhl_depth_zero_matches_consecutive_lookahead() {
        for n in 1..=7 {
            let mut deck: Vec<_> = (1..=n).collect();
            permutations(&mut deck, 0, &mut |permutation| {
                let consecutive =
                    solve(Algorithm::TwoKPartitionLookaheadSelection(1), permutation).unwrap();
                let depth_zero = solve(
                    Algorithm::DepthLimitedRhlTwoKPartitionLookaheadSelectionExperimental(1, 0),
                    permutation,
                )
                .unwrap();
                assert_eq!(depth_zero.plan, consecutive.plan);
                validate_sort_plan(permutation, &depth_zero.plan).unwrap();
            });
        }
    }

    #[test]
    fn depth_limited_rhl_sorts_and_never_loses_to_consecutive_lookahead() {
        let mut found_strict_improvement = false;
        for n in 2..=7 {
            let mut deck: Vec<_> = (1..=n).collect();
            permutations(&mut deck, 0, &mut |permutation| {
                let consecutive =
                    solve(Algorithm::TwoKPartitionLookaheadSelection(1), permutation).unwrap();
                for depth in 1..=4 {
                    let result = solve(
                        Algorithm::DepthLimitedRhlTwoKPartitionLookaheadSelectionExperimental(
                            1, depth,
                        ),
                        permutation,
                    )
                    .unwrap();
                    validate_sort_plan(permutation, &result.plan).unwrap();
                    assert!(
                        result.cost() <= consecutive.cost(),
                        "depth {depth} cost {} exceeded consecutive cost {} on {permutation:?}",
                        result.cost(),
                        consecutive.cost()
                    );
                    found_strict_improvement |= result.cost() < consecutive.cost();
                }
            });
        }
        assert!(found_strict_improvement);
    }

    #[test]
    fn depth_limited_value_matches_simple_reference_on_small_leaves() {
        for n in 2..=6 {
            let mut cards: Vec<_> = (1..=n).collect();
            permutations(&mut cards, 0, &mut |permutation| {
                for cut in 0..=n {
                    let state = State {
                        a: permutation[..cut].to_vec(),
                        d: Vec::new(),
                        b: permutation[cut..].to_vec(),
                    };
                    let source = if state.a.contains(&n) {
                        StackId::A
                    } else {
                        StackId::B
                    };
                    let partial = DepthLimitedState {
                        state,
                        low: 1,
                        current: n,
                        source,
                        destination: opposite_endpoint(source),
                        held: 0,
                        next_capture: n - 1,
                    };
                    for depth in 0..=5 {
                        let mut planner = DepthLimitedRhlPlanner::default();
                        let memoized = planner.depth_value(partial.clone(), depth).unwrap();
                        let reference = reference_depth_value(partial.clone(), depth).unwrap();
                        assert_eq!(
                            memoized, reference,
                            "value mismatch for n={n}, cut={cut}, depth={depth}, permutation={permutation:?}"
                        );
                    }
                }
            });
        }
    }

    #[test]
    fn isolated_depth_limited_selection_trace_records_stable_d_shape() {
        for n in 1..=6 {
            let mut cards: Vec<_> = (1..=n).collect();
            permutations(&mut cards, 0, &mut |permutation| {
                let trace = trace_depth_limited_selection_from_a(permutation, 7).unwrap();
                assert_eq!(
                    trace.cost,
                    trace
                        .steps
                        .iter()
                        .map(|step| step.primitive_cost)
                        .sum::<usize>()
                );
                assert_eq!(trace.steps.last().unwrap().state, State::goal(n));
                for step in &trace.steps {
                    let expected_suffix: Vec<_> = (step.current + 1..=n).collect();
                    assert_eq!(step.state.d.len(), step.held + expected_suffix.len());
                    assert_eq!(&step.state.d[step.held..], expected_suffix);
                }
            });
        }

        let counterexample =
            trace_depth_limited_selection_from_a(&[1, 4, 2, 3, 5, 6, 7], 7).unwrap();
        assert!(counterexample
            .steps
            .iter()
            .any(|step| step.state.d == [6, 4]));
    }

    #[test]
    fn depth_limited_selection_trace_accepts_arbitrary_configurations() {
        let midpass = State::new(vec![7], vec![6, 4], vec![5, 3, 2, 1]).unwrap();
        let inferred = DepthLimitedSelectionTraceConfig::inferred(midpass);
        assert_eq!(inferred.current, 7);
        assert_eq!(inferred.held, 2);
        assert_eq!(inferred.next_capture, 5);
        let trace = trace_depth_limited_selection(inferred, 7).unwrap();
        assert_eq!(trace.steps[0].state.d, [6, 4]);
        assert_eq!(trace.steps.last().unwrap().state, State::goal(7));

        let on_b = State::new(Vec::new(), Vec::new(), vec![2, 5, 1, 4, 3]).unwrap();
        let trace =
            trace_depth_limited_selection(DepthLimitedSelectionTraceConfig::inferred(on_b), 7)
                .unwrap();
        assert_eq!(trace.steps.last().unwrap().state, State::goal(5));

        let bucket = State::new(vec![6, 5, 4, 1, 2, 3], vec![7], Vec::new()).unwrap();
        let trace = trace_depth_limited_selection(
            DepthLimitedSelectionTraceConfig {
                state: bucket,
                low: 4,
                current: 6,
                held: 0,
                next_capture: 5,
            },
            7,
        )
        .unwrap();
        assert_eq!(trace.low, 4);
        assert_eq!(
            trace.steps.last().unwrap().state,
            State::new(vec![1, 2, 3], vec![4, 5, 6, 7], Vec::new()).unwrap()
        );
        assert_eq!(trace.steps.last().unwrap().current, 3);

        let trapped = State::new(vec![3], vec![2], vec![1]).unwrap();
        assert!(trace_depth_limited_selection(
            DepthLimitedSelectionTraceConfig {
                state: trapped,
                low: 1,
                current: 3,
                held: 0,
                next_capture: 2,
            },
            7,
        )
        .is_err());
    }

    #[test]
    fn target_block_algebra_matches_primitive_execution() {
        for n in 2..=6 {
            let mut cards: Vec<_> = (1..=n).collect();
            permutations(&mut cards, 0, &mut |permutation| {
                let active = ActiveBucketState {
                    a: permutation.to_vec(),
                    b: Vec::new(),
                };
                for (decision, expected) in TargetBlockDpPlanner::candidates(&active, n) {
                    let state = State::new(permutation.to_vec(), Vec::new(), Vec::new()).unwrap();
                    let mut machine = Machine::from_state(state);
                    apply_target_block_decision(&mut machine, n, decision).unwrap();
                    let actual = IncrementalRhlPlanner::from_machine(&machine, 1, n - 1);
                    assert_eq!(actual, expected, "n={n}, permutation={permutation:?}");
                }
            });
        }
    }

    #[test]
    fn target_block_dp_sorts_every_a_stack_permutation_through_seven() {
        for n in 0..=7 {
            let mut cards: Vec<_> = (1..=n).collect();
            permutations(&mut cards, 0, &mut |permutation| {
                let result = target_block_dp_selection_from_a(permutation).unwrap();
                let start = State::new(permutation.to_vec(), Vec::new(), Vec::new()).unwrap();
                assert_eq!(crate::replay(&start, &result.plan).unwrap(), State::goal(n));
            });
        }

        let captured = target_block_dp_selection_from_a(&[1, 4, 2, 3, 5, 6, 7]).unwrap();
        assert_eq!(captured.plan.len(), 19);
        let excluded = target_block_dp_selection_from_a(&[4, 2, 6, 1, 5, 7, 3]).unwrap();
        assert_eq!(excluded.plan.len(), 25);
    }

    #[test]
    fn target_block_dp_algorithm_sorts_standard_inputs() {
        let algorithms = [
            Algorithm::TargetBlockRolloutSelectionExperimental,
            Algorithm::TargetBlockRolloutTwoKPartitionSelectionExperimental(1),
            Algorithm::TargetBlockRolloutTwoKPartitionSelectionExperimental(2),
        ];
        for n in 0..=7 {
            let mut deck: Vec<_> = (1..=n).collect();
            permutations(&mut deck, 0, &mut |permutation| {
                for algorithm in algorithms {
                    let result = solve(algorithm, permutation).unwrap();
                    validate_sort_plan(permutation, &result.plan).unwrap();
                }
            });
        }
    }

    #[test]
    fn perfect_selection_sorts_and_never_loses_to_consecutive_lookahead() {
        let mut found_strict_improvement = false;
        for n in 2..=7 {
            let mut deck: Vec<_> = (1..=n).collect();
            permutations(&mut deck, 0, &mut |permutation| {
                let consecutive =
                    solve(Algorithm::TwoKPartitionLookaheadSelection(1), permutation).unwrap();
                let perfect =
                    solve(Algorithm::TwoKPartitioningPerfectSelection(1), permutation).unwrap();
                validate_sort_plan(permutation, &perfect.plan).unwrap();
                assert!(
                    perfect.cost() <= consecutive.cost(),
                    "perfect cost {} exceeded consecutive cost {} on {permutation:?}",
                    perfect.cost(),
                    consecutive.cost()
                );
                found_strict_improvement |= perfect.cost() < consecutive.cost();
            });
        }
        assert!(found_strict_improvement);
    }

    #[test]
    fn fixed_cost_regressions_at_52() {
        let deck: Vec<_> = (1..=52).rev().collect();
        assert_eq!(solve(Algorithm::Merge, &deck).unwrap().cost(), 1200);
        assert_eq!(solve(Algorithm::MsbRadix, &deck).unwrap().cost(), 880);
        assert_eq!(solve(Algorithm::LsbRadix, &deck).unwrap().cost(), 624);
        assert_eq!(solve(Algorithm::Natural, &deck).unwrap().cost(), 624);
        assert_eq!(
            solve(Algorithm::HuTuckerNaturalMerge, &deck)
                .unwrap()
                .cost(),
            600
        );
        assert_eq!(
            solve(Algorithm::SignedNaturalExperimental, &deck)
                .unwrap()
                .cost(),
            204
        );
        assert_eq!(solve(Algorithm::SplitMerge, &deck).unwrap().cost(), 624);
        assert_eq!(
            solve(Algorithm::ReversingSplitMergeExperimental, &deck)
                .unwrap()
                .cost(),
            206
        );
    }

    #[test]
    fn sorted_input_is_free() {
        let deck: Vec<_> = (1..=52).collect();
        for algorithm in Algorithm::ALL {
            assert!(solve(algorithm, &deck).unwrap().plan.is_empty());
        }
    }

    #[test]
    fn hu_tucker_tree_uses_the_minimum_alphabetic_cost() {
        fn brute_force_cost(weights: &[usize]) -> usize {
            if weights.len() <= 1 {
                return 0;
            }
            let root_cost: usize = weights.iter().sum();
            root_cost
                + (1..weights.len())
                    .map(|split| {
                        brute_force_cost(&weights[..split]) + brute_force_cost(&weights[split..])
                    })
                    .min()
                    .expect("nontrivial interval has a split")
        }

        fn weighted_path_length(tree: &AlphabeticMergeTree, depth: usize) -> usize {
            match tree {
                AlphabeticMergeTree::Leaf(size) => size * depth,
                AlphabeticMergeTree::Node { upper, lower, .. } => {
                    weighted_path_length(upper, depth + 1) + weighted_path_length(lower, depth + 1)
                }
            }
        }

        for weights in [
            vec![1],
            vec![1, 1],
            vec![1, 2, 3],
            vec![8, 1, 1, 5],
            vec![1, 4, 2, 7, 1, 3],
        ] {
            let tree = optimal_alphabetic_tree(&weights);
            assert_eq!(weighted_path_length(&tree, 0), brute_force_cost(&weights));
        }
    }
}
