//! Interactive tracing and exhaustive auditing for isolated depth-limited selection.

use std::collections::{HashMap, HashSet, VecDeque};
use std::process::ExitCode;

use three_stack_shuffle::algorithms::{
    target_block_dp_selection_from_a, target_block_rollout_selection_from_a,
    trace_depth_limited_selection, trace_depth_limited_selection_from_a,
    DepthLimitedSelectionTrace, DepthLimitedSelectionTraceConfig, DepthLimitedSelectionTraceEvent,
};
use three_stack_shuffle::search::ReverseBfs;
use three_stack_shuffle::{Move, StackId, State};

const HELP: &str = "\
Usage:
  selection-diagnostic --trace CARD,... [--depth D] [--optimal | --target-block]
  selection-diagnostic [--a CARD,...] [--d CARD,...] [--b CARD,...]
    [--low L] [--current C] [--held H] [--next-capture X] [--depth D] [--optimal]
  selection-diagnostic --audit [--n N] [--depth D] [--examples COUNT]
    [--optimal | --target-block]

Cards and displayed stacks are top-to-bottom. Use '-' for an empty stack.
For an arbitrary configuration, omitted stacks are empty. The default current
target, held count, and next capture are inferred from D's maximal goal suffix
and the staged prefix above it; any of them may be overridden.

Examples:
  cargo run --release --bin selection-diagnostic -- \\
    --trace 1,4,2,3,5,6,7 --depth 7
  cargo run --release --bin selection-diagnostic -- \\
    --a 7 --d 6,4 --b 5,3,2,1 --depth 7
  cargo run --release --bin selection-diagnostic -- \\
    --trace 1,4,2,3,5,6,7 --optimal
  cargo run --release --bin selection-diagnostic -- \\
    --audit --n 7 --depth 7 --examples 5
  cargo run --release --bin selection-diagnostic -- \\
    --audit --optimal --n 7 --examples 5
  cargo run --release --bin selection-diagnostic -- \\
    --audit --target-block --n 7 --examples 5
";

fn value<'a>(args: &'a [String], flag: &str) -> Result<Option<&'a str>, String> {
    let Some(index) = args.iter().position(|arg| arg == flag) else {
        return Ok(None);
    };
    args.get(index + 1)
        .map(String::as_str)
        .map(Some)
        .ok_or_else(|| format!("missing value after {flag}"))
}

fn usize_value(args: &[String], flag: &str, default: usize) -> Result<usize, String> {
    value(args, flag)?.map_or(Ok(default), |text| {
        text.parse()
            .map_err(|_| format!("invalid integer after {flag}: {text}"))
    })
}

fn optional_usize_value(args: &[String], flag: &str) -> Result<Option<usize>, String> {
    value(args, flag)?
        .map(|text| {
            text.parse()
                .map_err(|_| format!("invalid integer after {flag}: {text}"))
        })
        .transpose()
}

fn parse_stack(text: &str, flag: &str) -> Result<Vec<usize>, String> {
    if text.is_empty() || text == "-" {
        return Ok(Vec::new());
    }
    let cards: Result<Vec<_>, _> = text
        .split(',')
        .map(|card| {
            card.trim()
                .parse::<usize>()
                .map_err(|_| format!("invalid card in {flag}: {card}"))
        })
        .collect();
    cards
}

fn parse_deck(text: &str) -> Result<Vec<usize>, String> {
    let cards = parse_stack(text, "--trace deck")?;
    if cards.is_empty() {
        Err("--trace requires at least one card".into())
    } else {
        Ok(cards)
    }
}

fn event_name(event: &DepthLimitedSelectionTraceEvent) -> String {
    match event {
        DepthLimitedSelectionTraceEvent::Start => "start".into(),
        DepthLimitedSelectionTraceEvent::PlaceTargets { first, count } => {
            let last = first + 1 - count;
            if *count == 1 {
                format!("place {first}")
            } else {
                format!("place {first}..{last}")
            }
        }
        DepthLimitedSelectionTraceEvent::Stage { card, greedy } => {
            format!("stage {card}{}", if *greedy { "" } else { " (nongreedy)" })
        }
        DepthLimitedSelectionTraceEvent::Bypass { card, greedy } => {
            format!("bypass {card}{}", if *greedy { "" } else { " (nongreedy)" })
        }
    }
}

fn print_trace(trace: &DepthLimitedSelectionTrace) {
    let initial = &trace.steps[0];
    println!(
        "selection trace: n={} depth={} low={} current={} held={} next={}",
        initial.state.len(),
        trace.depth,
        trace.low,
        initial.current,
        initial.held,
        initial.next_capture
    );
    let mut cumulative = 0;
    for (index, step) in trace.steps.iter().enumerate() {
        cumulative += step.primitive_cost;
        println!(
            "{index:>3}  +{:>2}  total={cumulative:>3}  current={:>2}  held={:>2}  next={:>2}  {}",
            step.primitive_cost,
            step.current,
            step.held,
            step.next_capture,
            event_name(&step.event)
        );
        println!(
            "     A={:?}  D={:?}  B={:?}",
            step.state.a, step.state.d, step.state.b
        );
    }
    println!("primitive moves: {}", trace.cost);
}

#[derive(Clone, Copy)]
enum PrefixOrder {
    Ascending,
    Descending,
    Either,
}

impl PrefixOrder {
    fn label(self) -> &'static str {
        match self {
            Self::Ascending => "ascending contiguous prefix (inverted onto D)",
            Self::Descending => "descending contiguous prefix",
            Self::Either => "contiguous prefix in either direction",
        }
    }
}

fn consecutive(values: &[usize], order: PrefixOrder) -> bool {
    let ascending = || values.windows(2).all(|pair| pair[1] == pair[0] + 1);
    let descending = || values.windows(2).all(|pair| pair[0] == pair[1] + 1);
    match order {
        PrefixOrder::Ascending => ascending(),
        PrefixOrder::Descending => descending(),
        PrefixOrder::Either => ascending() || descending(),
    }
}

fn d_has_contiguous_prefix_and_goal_suffix(state: &State, order: PrefixOrder) -> bool {
    let n = state.len();
    (0..=state.d.len()).any(|split| {
        let suffix = &state.d[split..];
        let expected_start = n + 1 - suffix.len();
        consecutive(&state.d[..split], order) && suffix.iter().copied().eq(expected_start..=n)
    })
}

struct OptimalDag {
    states: Vec<State>,
    edges: usize,
    parent: HashMap<State, (State, Move)>,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
struct ParkingTracker {
    /// Cards whose current D residence began with a move during this sort.
    active_landed: u64,
    /// Card that landed on D in the immediately preceding primitive move.
    just_landed: Option<usize>,
    /// Cards that have completed one nonfinal, non-passing D residence.
    parked_once: u64,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct ParkingNode {
    state: State,
    tracker: ParkingTracker,
}

struct ParkingAudit {
    preserving_goal: Option<ParkingNode>,
    violation: Option<(ParkingNode, Move, State, usize)>,
    parent: HashMap<ParkingNode, (ParkingNode, Move)>,
    augmented_states: usize,
}

fn top_card(state: &State, stack: StackId) -> usize {
    match stack {
        StackId::A => state.a[0],
        StackId::D => state.d[0],
        StackId::B => state.b[0],
    }
}

fn advance_parking(
    state: &State,
    movement: Move,
    mut tracker: ParkingTracker,
) -> Result<ParkingTracker, usize> {
    let (source, destination) = movement.endpoints();
    let card = top_card(state, source);
    let bit = 1_u64 << (card - 1);
    let immediate_departure = source == StackId::D && tracker.just_landed == Some(card);

    if source == StackId::D && tracker.active_landed & bit != 0 {
        tracker.active_landed &= !bit;
        if !immediate_departure {
            if tracker.parked_once & bit != 0 {
                return Err(card);
            }
            tracker.parked_once |= bit;
        }
    }

    tracker.just_landed = None;
    if destination == StackId::D {
        tracker.active_landed |= bit;
        tracker.just_landed = Some(card);
    }
    Ok(tracker)
}

fn parking_audit(database: &ReverseBfs, start: &State) -> ParkingAudit {
    assert!(start.len() <= 64, "parking masks support at most 64 cards");
    let start_node = ParkingNode {
        state: start.clone(),
        tracker: ParkingTracker::default(),
    };
    let mut seen = HashSet::from([start_node.clone()]);
    let mut queue = VecDeque::from([start_node]);
    let mut parent = HashMap::new();
    let mut preserving_goal = None;
    let mut violation = None;

    while let Some(node) = queue.pop_front() {
        if database.distance(&node.state) == Some(0) && preserving_goal.is_none() {
            preserving_goal = Some(node.clone());
        }
        for (child, movement) in optimal_children(database, &node.state) {
            match advance_parking(&node.state, movement, node.tracker) {
                Ok(tracker) => {
                    let child_node = ParkingNode {
                        state: child,
                        tracker,
                    };
                    if seen.insert(child_node.clone()) {
                        parent.insert(child_node.clone(), (node.clone(), movement));
                        queue.push_back(child_node);
                    }
                }
                Err(card) => {
                    if violation.is_none() {
                        violation = Some((node.clone(), movement, child, card));
                    }
                }
            }
        }
    }

    ParkingAudit {
        preserving_goal,
        violation,
        parent,
        augmented_states: seen.len(),
    }
}

fn parking_prefix_plan(
    start: &State,
    node: &ParkingNode,
    parent: &HashMap<ParkingNode, (ParkingNode, Move)>,
) -> Vec<Move> {
    let mut current = node.clone();
    let mut plan = Vec::new();
    while current.state != *start || current.tracker != ParkingTracker::default() {
        let (previous, movement) = parent
            .get(&current)
            .expect("reachable augmented state has a parent");
        plan.push(*movement);
        current = previous.clone();
    }
    plan.reverse();
    plan
}

fn parked_cards(mask: u64, n: usize) -> Vec<usize> {
    (1..=n)
        .filter(|&card| mask & (1_u64 << (card - 1)) != 0)
        .collect()
}

fn optimal_children(database: &ReverseBfs, state: &State) -> Vec<(State, Move)> {
    let distance = database
        .distance(state)
        .expect("state belongs to the complete reverse BFS");
    if distance == 0 {
        return Vec::new();
    }
    state
        .neighbors()
        .into_iter()
        .filter(|(child, _)| database.distance(child) == Some(distance - 1))
        .collect()
}

fn optimal_dag(database: &ReverseBfs, start: &State) -> OptimalDag {
    let mut states = Vec::new();
    let mut edges = 0;
    let mut parent = HashMap::new();
    let mut seen = HashSet::from([start.clone()]);
    let mut queue = VecDeque::from([start.clone()]);
    while let Some(state) = queue.pop_front() {
        states.push(state.clone());
        for (child, movement) in optimal_children(database, &state) {
            edges += 1;
            if seen.insert(child.clone()) {
                parent.insert(child.clone(), (state.clone(), movement));
                queue.push_back(child);
            }
        }
    }
    OptimalDag {
        states,
        edges,
        parent,
    }
}

fn has_property_preserving_completion(
    database: &ReverseBfs,
    state: &State,
    order: PrefixOrder,
    memo: &mut HashMap<State, bool>,
) -> bool {
    if let Some(&answer) = memo.get(state) {
        return answer;
    }
    let answer = d_has_contiguous_prefix_and_goal_suffix(state, order)
        && (database.distance(state) == Some(0)
            || optimal_children(database, state)
                .into_iter()
                .any(|(child, _)| {
                    has_property_preserving_completion(database, &child, order, memo)
                }));
    memo.insert(state.clone(), answer);
    answer
}

fn property_preserving_plan(
    database: &ReverseBfs,
    start: &State,
    order: PrefixOrder,
) -> Option<Vec<Move>> {
    let mut memo = HashMap::new();
    if !has_property_preserving_completion(database, start, order, &mut memo) {
        return None;
    }
    let mut state = start.clone();
    let mut plan = Vec::new();
    while database.distance(&state) != Some(0) {
        let (child, movement) = optimal_children(database, &state)
            .into_iter()
            .find(|(child, _)| {
                has_property_preserving_completion(database, child, order, &mut memo)
            })
            .expect("a property-preserving child was established by the dynamic program");
        plan.push(movement);
        state = child;
    }
    Some(plan)
}

fn optimal_path_count_capped(
    database: &ReverseBfs,
    state: &State,
    memo: &mut HashMap<State, u8>,
) -> u8 {
    if let Some(&count) = memo.get(state) {
        return count;
    }
    let count = if database.distance(state) == Some(0) {
        1
    } else {
        optimal_children(database, state)
            .into_iter()
            .map(|(child, _)| optimal_path_count_capped(database, &child, memo))
            .fold(0_u8, |total, count| total.saturating_add(count).min(2))
    };
    memo.insert(state.clone(), count);
    count
}

fn plan_through_state(
    database: &ReverseBfs,
    start: &State,
    state: &State,
    parent: &HashMap<State, (State, Move)>,
) -> Vec<Move> {
    let mut current = state.clone();
    let mut prefix = Vec::new();
    while current != *start {
        let (previous, movement) = parent
            .get(&current)
            .expect("reachable optimal-DAG state has a parent");
        prefix.push(*movement);
        current = previous.clone();
    }
    prefix.reverse();
    prefix.extend(
        database
            .plan(state)
            .expect("optimal-DAG state has an optimal goal plan"),
    );
    prefix
}

fn format_plan(plan: &[Move]) -> String {
    plan.iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

fn print_optimal_audit(start: &State) -> Result<(), String> {
    start.validate().map_err(|error| error.to_string())?;
    if start.len() > 64 {
        return Err("temporary-parking audit supports at most 64 cards".into());
    }
    let database = ReverseBfs::build(start.len());
    let distance = database
        .distance(start)
        .ok_or_else(|| "initial state is absent from the complete state graph".to_string())?;
    let dag = optimal_dag(&database, start);
    let optimal_path_count = optimal_path_count_capped(&database, start, &mut HashMap::new());
    println!("exact optimal-sort audit: n={}", start.len());
    println!("start: A={:?} D={:?} B={:?}", start.a, start.d, start.b);
    println!("complete states:             {}", database.len());
    println!("optimal distance:            {distance}");
    println!(
        "optimal sorts:               {}",
        if optimal_path_count == 1 {
            "exactly one"
        } else {
            "more than one"
        }
    );
    println!("reachable optimal-DAG states: {}", dag.states.len());
    println!("reachable optimal-DAG edges:  {}", dag.edges);

    for order in [
        PrefixOrder::Ascending,
        PrefixOrder::Descending,
        PrefixOrder::Either,
    ] {
        let preserving = property_preserving_plan(&database, start, order);
        let first_violation = dag
            .states
            .iter()
            .find(|state| !d_has_contiguous_prefix_and_goal_suffix(state, order));
        println!("property: {}", order.label());
        println!(
            "  preserving optimal sort exists: {}",
            if preserving.is_some() { "yes" } else { "no" }
        );
        println!(
            "  violating optimal sort exists:  {}",
            if first_violation.is_some() {
                "yes"
            } else {
                "no"
            }
        );
        match (preserving.is_some(), first_violation.is_some()) {
            (true, true) => println!("  conclusion: optimal sorts of both kinds exist"),
            (true, false) => println!("  conclusion: every optimal sort preserves it"),
            (false, true) => println!("  conclusion: every optimal sort violates it"),
            (false, false) => unreachable!("the optimal DAG contains at least one path"),
        }
        if let Some(plan) = preserving {
            println!("  preserving witness: {}", format_plan(&plan));
        }
        if let Some(violation) = first_violation {
            let plan = plan_through_state(&database, start, violation, &dag.parent);
            let violation_step = distance
                - database
                    .distance(violation)
                    .expect("optimal-DAG state has a goal distance");
            println!(
                "  first violation at step {violation_step}, D={:?}; witness: {}",
                violation.d,
                format_plan(&plan)
            );
        }
    }

    let parking = parking_audit(&database, start);
    println!("temporary-parking condition:");
    println!(
        "  nonviolating augmented states: {}",
        parking.augmented_states
    );
    println!(
        "  preserving optimal sort exists: {}",
        if parking.preserving_goal.is_some() {
            "yes"
        } else {
            "no"
        }
    );
    println!(
        "  violating optimal sort exists:  {}",
        if parking.violation.is_some() {
            "yes"
        } else {
            "no"
        }
    );
    match (
        parking.preserving_goal.is_some(),
        parking.violation.is_some(),
    ) {
        (true, true) => println!("  conclusion: optimal sorts of both kinds exist"),
        (true, false) => println!("  conclusion: every optimal sort parks each card at most once"),
        (false, true) => {
            println!("  conclusion: every optimal sort parks some card more than once")
        }
        (false, false) => unreachable!("the optimal DAG contains at least one path"),
    }
    if let Some(goal) = &parking.preserving_goal {
        let plan = parking_prefix_plan(start, goal, &parking.parent);
        println!(
            "  preserving witness parked cards: {:?}; plan: {}",
            parked_cards(goal.tracker.parked_once, start.len()),
            format_plan(&plan)
        );
    }
    if let Some((node, movement, child, card)) = &parking.violation {
        let mut plan = parking_prefix_plan(start, node, &parking.parent);
        plan.push(*movement);
        plan.extend(
            database
                .plan(child)
                .expect("violating child has an optimal goal plan"),
        );
        println!(
            "  violation witness parks card {card} a second time: {}",
            format_plan(&plan)
        );
    }
    Ok(())
}

fn print_target_block_comparison(cards: &[usize]) -> Result<(), String> {
    let global = target_block_dp_selection_from_a(cards).map_err(|error| error.to_string())?;
    let rollout =
        target_block_rollout_selection_from_a(cards).map_err(|error| error.to_string())?;
    let start =
        State::new(cards.to_vec(), Vec::new(), Vec::new()).map_err(|error| error.to_string())?;
    let database = ReverseBfs::build(cards.len());
    let optimum = database
        .distance(&start)
        .ok_or_else(|| "initial state is absent from the complete state graph".to_string())?;
    println!("consecutive-target block comparison: n={}", cards.len());
    println!("start A={cards:?}");
    println!("optimal: {optimum}");
    for (label, result) in [("global physical-state DP", global), ("rollout", rollout)] {
        println!("{label}:");
        println!("  cost:               {}", result.plan.len());
        println!("  gap:                {}", result.plan.len() - optimum);
        println!("  states:             {}", result.stats.states);
        println!("  transitions:        {}", result.stats.transitions);
        println!("  maximum candidates: {}", result.stats.max_candidates);
        println!("  cache hits:         {}", result.stats.cache_hits);
        println!("  forced targets:     {}", result.stats.forced_targets);
        println!("  plan:               {}", format_plan(&result.plan));
    }
    Ok(())
}

fn permutations(
    values: &mut [usize],
    start: usize,
    visit: &mut impl FnMut(&[usize]) -> Result<(), String>,
) -> Result<(), String> {
    if start == values.len() {
        return visit(values);
    }
    for index in start..values.len() {
        values.swap(start, index);
        permutations(values, start + 1, visit)?;
        values.swap(start, index);
    }
    Ok(())
}

fn run_optimal_parking_audit(n: usize, example_limit: usize) -> Result<(), String> {
    if n == 0 {
        return Err("--audit --optimal requires n >= 1".into());
    }
    if n > 64 {
        return Err("temporary-parking audit supports at most 64 cards".into());
    }

    let database = ReverseBfs::build(n);
    let mut cards: Vec<_> = (1..=n).collect();
    let mut permutations_checked = 0;
    let mut every_optimum_preserves = 0;
    let mut mixed = 0;
    let mut every_optimum_violates = 0;
    let mut largest_augmented_graph = 0;
    let mut mixed_examples = Vec::new();
    let mut unavoidable_examples = Vec::new();

    permutations(&mut cards, 0, &mut |deck| {
        permutations_checked += 1;
        let start =
            State::new(deck.to_vec(), Vec::new(), Vec::new()).map_err(|error| error.to_string())?;
        let parking = parking_audit(&database, &start);
        largest_augmented_graph = largest_augmented_graph.max(parking.augmented_states);
        match (
            parking.preserving_goal.is_some(),
            parking.violation.as_ref(),
        ) {
            (true, None) => every_optimum_preserves += 1,
            (true, Some((_, _, _, card))) => {
                mixed += 1;
                if mixed_examples.len() < example_limit {
                    mixed_examples.push(format!("deck={deck:?}, repeatedly parked card={card}"));
                }
            }
            (false, Some((_, _, _, card))) => {
                every_optimum_violates += 1;
                if unavoidable_examples.len() < example_limit {
                    unavoidable_examples
                        .push(format!("deck={deck:?}, repeatedly parked card={card}"));
                }
            }
            (false, None) => unreachable!("every initial state has at least one optimal sort"),
        }
        Ok(())
    })?;

    println!("exact optimal temporary-parking audit: n={n}");
    println!("complete states:                        {}", database.len());
    println!("A-stack permutations:                   {permutations_checked}");
    println!("every optimal sort preserves condition: {every_optimum_preserves}");
    println!("both preserving and violating optima:   {mixed}");
    println!("every optimal sort violates condition:  {every_optimum_violates}");
    println!("largest augmented optimal graph:         {largest_augmented_graph}");
    if !mixed_examples.is_empty() {
        println!("mixed examples:");
        for example in mixed_examples {
            println!("  {example}");
        }
    }
    if !unavoidable_examples.is_empty() {
        println!("unavoidable-violation examples:");
        for example in unavoidable_examples {
            println!("  {example}");
        }
    }
    Ok(())
}

fn run_target_block_audit(n: usize, example_limit: usize) -> Result<(), String> {
    if n == 0 {
        return Err("--audit --target-block requires n >= 1".into());
    }
    let database = ReverseBfs::build(n);
    let mut cards: Vec<_> = (1..=n).collect();
    let mut count = 0;
    let mut exact = 0;
    let mut rollout_exact = 0;
    let mut total_cost = 0_u128;
    let mut rollout_total_cost = 0_u128;
    let mut total_optimum = 0_u128;
    let mut total_states = 0_u128;
    let mut total_transitions = 0_u128;
    let mut max_gap = 0;
    let mut rollout_max_gap = 0;
    let mut max_states = 0;
    let mut max_transitions = 0;
    let mut worst_examples = Vec::new();

    permutations(&mut cards, 0, &mut |deck| {
        count += 1;
        let start =
            State::new(deck.to_vec(), Vec::new(), Vec::new()).map_err(|error| error.to_string())?;
        let optimum = database
            .distance(&start)
            .expect("complete reverse BFS contains every valid state");
        let result = target_block_dp_selection_from_a(deck).map_err(|error| error.to_string())?;
        let rollout =
            target_block_rollout_selection_from_a(deck).map_err(|error| error.to_string())?;
        let cost = result.plan.len();
        let rollout_cost = rollout.plan.len();
        if cost == optimum {
            exact += 1;
        }
        if rollout_cost == optimum {
            rollout_exact += 1;
        }
        let gap = cost - optimum;
        if gap > max_gap {
            max_gap = gap;
            worst_examples.clear();
        }
        if gap == max_gap && worst_examples.len() < example_limit {
            worst_examples.push(format!(
                "deck={deck:?}, cost={cost}, optimal={optimum}, states={}, transitions={}",
                result.stats.states, result.stats.transitions
            ));
        }
        total_cost += cost as u128;
        rollout_total_cost += rollout_cost as u128;
        total_optimum += optimum as u128;
        total_states += result.stats.states as u128;
        total_transitions += result.stats.transitions as u128;
        max_states = max_states.max(result.stats.states);
        max_transitions = max_transitions.max(result.stats.transitions);
        rollout_max_gap = rollout_max_gap.max(rollout_cost - optimum);
        Ok(())
    })?;

    println!("consecutive-target block DP exhaustive audit: n={n}");
    println!("complete states:          {}", database.len());
    println!("A-stack permutations:     {count}");
    println!("global DP exactly optimal:          {exact}");
    println!("rollout exactly optimal:            {rollout_exact}");
    println!(
        "global DP mean cost:                {:.6}",
        total_cost as f64 / count as f64
    );
    println!(
        "rollout mean cost:                  {:.6}",
        rollout_total_cost as f64 / count as f64
    );
    println!(
        "mean optimal:             {:.6}",
        total_optimum as f64 / count as f64
    );
    println!(
        "global DP mean additive gap:        {:.6}",
        (total_cost - total_optimum) as f64 / count as f64
    );
    println!(
        "rollout mean additive gap:          {:.6}",
        (rollout_total_cost - total_optimum) as f64 / count as f64
    );
    println!("global DP maximum additive gap:     {max_gap}");
    println!("rollout maximum additive gap:       {rollout_max_gap}");
    println!(
        "mean DP states:           {:.3}",
        total_states as f64 / count as f64
    );
    println!("maximum DP states:        {max_states}");
    println!(
        "mean DP transitions:      {:.3}",
        total_transitions as f64 / count as f64
    );
    println!("maximum DP transitions:   {max_transitions}");
    if !worst_examples.is_empty() {
        println!("maximum-gap examples:");
        for example in worst_examples {
            println!("  {example}");
        }
    }
    Ok(())
}

#[derive(Default)]
struct Audit {
    permutations: usize,
    stable_states: usize,
    decisions: usize,
    stages: usize,
    nongreedy_stages: usize,
    shape_violations: usize,
    reversed_stage_order_violations: usize,
    nonascending_rank_prefixes: usize,
    noneither_rank_prefixes: usize,
    split_stage_runs: usize,
    shape_examples: Vec<String>,
    stage_order_examples: Vec<String>,
    rank_examples: Vec<String>,
    split_run_examples: Vec<String>,
}

fn retain_example(examples: &mut Vec<String>, limit: usize, text: String) {
    if examples.len() < limit {
        examples.push(text);
    }
}

fn audit_trace(
    deck: &[usize],
    trace: &DepthLimitedSelectionTrace,
    limit: usize,
    audit: &mut Audit,
) {
    audit.permutations += 1;
    let n = deck.len();
    let mut staged = Vec::new();
    let mut saw_stage = false;
    let mut bypass_after_stage = false;

    for step in &trace.steps {
        audit.stable_states += 1;
        match &step.event {
            DepthLimitedSelectionTraceEvent::Start => {}
            DepthLimitedSelectionTraceEvent::PlaceTargets { .. } => {
                staged.clear();
                saw_stage = false;
                bypass_after_stage = false;
            }
            DepthLimitedSelectionTraceEvent::Stage { card, greedy } => {
                audit.decisions += 1;
                audit.stages += 1;
                audit.nongreedy_stages += usize::from(!greedy);
                if bypass_after_stage {
                    audit.split_stage_runs += 1;
                    retain_example(
                        &mut audit.split_run_examples,
                        limit,
                        format!(
                            "split stage run: deck={deck:?}, event={}, current={}, D={:?}",
                            event_name(&step.event),
                            step.current,
                            step.state.d
                        ),
                    );
                }
                staged.push(*card);
                saw_stage = true;
            }
            DepthLimitedSelectionTraceEvent::Bypass { .. } => {
                audit.decisions += 1;
                if saw_stage {
                    bypass_after_stage = true;
                }
            }
        }

        let expected_suffix: Vec<_> = (step.current + 1..=n).collect();
        let shape_ok = step.state.d.len() == step.held + expected_suffix.len()
            && step.state.d.get(step.held..) == Some(expected_suffix.as_slice());
        if !shape_ok {
            audit.shape_violations += 1;
            retain_example(
                &mut audit.shape_examples,
                limit,
                format!(
                    "D-shape violation: deck={deck:?}, event={}, current={}, held={}, D={:?}, expected suffix={expected_suffix:?}",
                    event_name(&step.event),
                    step.current,
                    step.held,
                    step.state.d
                ),
            );
            continue;
        }

        let prefix = &step.state.d[..step.held];
        if !prefix.iter().eq(staged.iter().rev()) {
            audit.reversed_stage_order_violations += 1;
            retain_example(
                &mut audit.stage_order_examples,
                limit,
                format!(
                    "stage-order violation: deck={deck:?}, event={}, staged={staged:?}, D={:?}",
                    event_name(&step.event),
                    step.state.d
                ),
            );
        }
        if !prefix.windows(2).all(|pair| pair[1] == pair[0] + 1) {
            audit.nonascending_rank_prefixes += 1;
        }
        let descending = prefix.windows(2).all(|pair| pair[0] == pair[1] + 1);
        let ascending = prefix.windows(2).all(|pair| pair[1] == pair[0] + 1);
        if !ascending && !descending {
            audit.noneither_rank_prefixes += 1;
            retain_example(
                &mut audit.rank_examples,
                limit,
                format!(
                    "non-rank-contiguous prefix: deck={deck:?}, event={}, current={}, D={:?}",
                    event_name(&step.event),
                    step.current,
                    step.state.d
                ),
            );
        }
    }
}

fn run_audit(n: usize, depth: usize, example_limit: usize) -> Result<(), String> {
    if n == 0 {
        return Err("--audit requires n >= 1".into());
    }
    let mut cards: Vec<_> = (1..=n).collect();
    let mut audit = Audit::default();
    permutations(&mut cards, 0, &mut |deck| {
        let trace =
            trace_depth_limited_selection_from_a(deck, depth).map_err(|error| error.to_string())?;
        audit_trace(deck, &trace, example_limit, &mut audit);
        Ok(())
    })?;

    println!("isolated selection audit: n={n} depth={depth}");
    println!(
        "permutations:                         {}",
        audit.permutations
    );
    println!(
        "stable states:                       {}",
        audit.stable_states
    );
    println!("blocker decisions:                   {}", audit.decisions);
    println!("stage decisions:                     {}", audit.stages);
    println!(
        "nongreedy stages:                    {}",
        audit.nongreedy_stages
    );
    println!(
        "D prefix + in-place suffix failures: {}",
        audit.shape_violations
    );
    println!(
        "reversed stage-order failures:       {}",
        audit.reversed_stage_order_violations
    );
    println!(
        "nonascending rank-prefix states:     {}",
        audit.nonascending_rank_prefixes
    );
    println!(
        "non-rank-contiguous prefix states:   {}",
        audit.noneither_rank_prefixes
    );
    println!(
        "split stage runs:                    {}",
        audit.split_stage_runs
    );
    for (label, examples) in [
        ("D-shape examples", audit.shape_examples),
        ("stage-order examples", audit.stage_order_examples),
        ("non-rank-contiguous examples", audit.rank_examples),
        ("split-stage-run examples", audit.split_run_examples),
    ] {
        if examples.is_empty() {
            continue;
        }
        println!("{label}:");
        for example in examples {
            println!("  {example}");
        }
    }
    Ok(())
}

fn stack_arg(args: &[String], flag: &str) -> Result<Vec<usize>, String> {
    value(args, flag)?.map_or(Ok(Vec::new()), |text| parse_stack(text, flag))
}

fn inferred_next_capture(current: usize, d: &[usize], held: usize) -> Result<usize, String> {
    if held > d.len() {
        return Err(format!(
            "--held {held} exceeds the {} cards supplied on D",
            d.len()
        ));
    }
    let mut next = current.saturating_sub(1);
    for &card in d[..held].iter().rev() {
        if card == next {
            next = next.saturating_sub(1);
        }
    }
    Ok(next)
}

fn configuration_state(args: &[String]) -> Result<State, String> {
    State::new(
        stack_arg(args, "--a")?,
        stack_arg(args, "--d")?,
        stack_arg(args, "--b")?,
    )
    .map_err(|error| error.to_string())
}

fn trace_configuration(
    args: &[String],
    depth: usize,
) -> Result<DepthLimitedSelectionTrace, String> {
    let state = configuration_state(args)?;
    let inferred = DepthLimitedSelectionTraceConfig::inferred(state);
    let low = optional_usize_value(args, "--low")?.unwrap_or(inferred.low);
    let current = optional_usize_value(args, "--current")?.unwrap_or(inferred.current);
    let held = optional_usize_value(args, "--held")?.unwrap_or(inferred.held);
    let next_capture = optional_usize_value(args, "--next-capture")?.map_or_else(
        || inferred_next_capture(current, &inferred.state.d, held),
        Ok,
    )?;
    trace_depth_limited_selection(
        DepthLimitedSelectionTraceConfig {
            state: inferred.state,
            low,
            current,
            held,
            next_capture,
        },
        depth,
    )
    .map_err(|error| error.to_string())
}

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args().collect();
    let run = || -> Result<(), String> {
        if args.iter().any(|arg| arg == "--help" || arg == "-h") {
            print!("{HELP}");
            return Ok(());
        }

        let depth = usize_value(&args, "--depth", 7)?;
        let trace_deck = value(&args, "--trace")?;
        let audit = args.iter().any(|arg| arg == "--audit");
        let optimal = args.iter().any(|arg| arg == "--optimal");
        let target_block = args.iter().any(|arg| arg == "--target-block");
        let configuration = ["--a", "--d", "--b"]
            .iter()
            .any(|flag| args.iter().any(|arg| arg == flag));
        let mode_count =
            usize::from(trace_deck.is_some()) + usize::from(audit) + usize::from(configuration);
        if mode_count != 1 {
            return Err(format!(
                "choose exactly one of --trace, --audit, or an A/D/B configuration\n\n{HELP}"
            ));
        }
        if optimal && target_block {
            return Err("choose at most one of --optimal and --target-block".into());
        }
        if let Some(deck) = trace_deck {
            let cards = parse_deck(deck)?;
            if target_block {
                return print_target_block_comparison(&cards);
            }
            if optimal {
                let state =
                    State::new(cards, Vec::new(), Vec::new()).map_err(|error| error.to_string())?;
                return print_optimal_audit(&state);
            }
            let trace = trace_depth_limited_selection_from_a(&cards, depth)
                .map_err(|error| error.to_string())?;
            print_trace(&trace);
            return Ok(());
        }
        if audit {
            let n = usize_value(&args, "--n", 7)?;
            let examples = usize_value(&args, "--examples", 5)?;
            if target_block {
                return run_target_block_audit(n, examples);
            }
            if optimal {
                return run_optimal_parking_audit(n, examples);
            }
            return run_audit(n, depth, examples);
        }
        if target_block {
            return Err("--target-block currently requires --trace or --audit".into());
        }
        if optimal {
            if ["--low", "--current", "--held", "--next-capture"]
                .iter()
                .any(|flag| args.iter().any(|arg| arg == flag))
            {
                return Err(
                    "selection metadata flags do not apply to an exact optimal sort".into(),
                );
            }
            return print_optimal_audit(&configuration_state(&args)?);
        }
        let trace = trace_configuration(&args, depth)?;
        print_trace(&trace);
        Ok(())
    };

    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("selection-diagnostic: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seven_card_counterexample_has_one_optimum_and_every_optimum_violates() {
        let start = State::new(vec![1, 4, 2, 3, 5, 6, 7], Vec::new(), Vec::new()).unwrap();
        let database = ReverseBfs::build(7);
        assert_eq!(database.len(), 181_440);
        assert_eq!(database.distance(&start), Some(19));
        assert_eq!(
            optimal_path_count_capped(&database, &start, &mut HashMap::new()),
            1
        );

        let dag = optimal_dag(&database, &start);
        assert_eq!(dag.states.len(), 20);
        assert_eq!(dag.edges, 19);
        for order in [
            PrefixOrder::Ascending,
            PrefixOrder::Descending,
            PrefixOrder::Either,
        ] {
            assert!(property_preserving_plan(&database, &start, order).is_none());
            assert!(dag
                .states
                .iter()
                .any(|state| !d_has_contiguous_prefix_and_goal_suffix(state, order)));
        }

        let parking = parking_audit(&database, &start);
        let goal = parking
            .preserving_goal
            .expect("the unique optimum obeys the parking condition");
        assert_eq!(parked_cards(goal.tracker.parked_once, 7), [4]);
        assert!(parking.violation.is_none());
    }

    #[test]
    fn seven_card_optimum_can_require_parking_one_card_twice() {
        let start = State::new(vec![4, 2, 6, 1, 5, 7, 3], Vec::new(), Vec::new()).unwrap();
        let database = ReverseBfs::build(7);
        assert_eq!(database.distance(&start), Some(19));
        assert_eq!(
            optimal_path_count_capped(&database, &start, &mut HashMap::new()),
            1
        );

        let parking = parking_audit(&database, &start);
        assert!(parking.preserving_goal.is_none());
        let (_, _, _, card) = parking
            .violation
            .expect("the unique optimum parks a card twice");
        assert_eq!(card, 6);
    }
}
