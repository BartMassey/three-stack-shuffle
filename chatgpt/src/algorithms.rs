//! Constructive sorting algorithms from `ALGORITHMS.md`.
//!
//! Every implementation drives [`Machine`]; none mutates a
//! stack directly. Experimental algorithms are explicitly marked in
//! [`Algorithm`] and use certified fallbacks when their pure phase rule is not
//! established by the specification.

use std::collections::BTreeMap;

use crate::macros::{move_cards, reverse_d, reverse_d_to_endpoint};
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
            Self::SignedNaturalExperimental | Self::ReversingSplitMergeExperimental
        )
    }

    /// Parses a stable command-line name.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        if let Some(k) = name.strip_prefix("2k-partition-lookahead-selection:") {
            return k
                .parse::<usize>()
                .ok()
                .filter(|&k| k > 0)
                .map(Self::TwoKPartitionLookaheadSelection);
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
    if matches!(algorithm, Algorithm::TwoKPartitionLookaheadSelection(0)) {
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
            partition_lookahead_selection(deck, algorithm, buckets)
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
    stats: &mut SortStats,
) -> Result<(), MachineError> {
    let card_count = high - low + 1;
    if bucket_count == 1 {
        return extract_with_lookahead(machine, high, low - 1, stats);
    }

    let lower_buckets = bucket_count / 2;
    let upper_buckets = bucket_count - lower_buckets;
    let lower_cards = lower_partition_size(card_count, bucket_count, lower_buckets);
    let split = low + lower_cards - 1;

    repartition_endpoint(machine, card_count, source, split)?;
    extract_partition_tree(machine, split + 1, high, upper_buckets, StackId::B, stats)?;
    extract_partition_tree(machine, low, split, lower_buckets, StackId::A, stats)
}

fn partition_lookahead_selection(
    deck: &[usize],
    algorithm: Algorithm,
    requested_buckets: usize,
) -> Result<SortResult, MachineError> {
    let mut machine = Machine::new(deck)?;
    let m = active_prefix(deck);
    let bucket_count = requested_buckets.min(m);
    debug_assert!(
        bucket_count >= 2,
        "non-sorted active prefixes have at least two cards"
    );
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
        &mut stats,
    )?;
    extract_partition_tree(
        &mut machine,
        1,
        lower_cards,
        lower_buckets,
        StackId::A,
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
    fn all_algorithms_sort_every_permutation_through_eight() {
        for n in 0..=8 {
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
