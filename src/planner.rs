//! An anytime, *inadmissible* local-search planner (docs/NOTES.md §I.6 #5). Two
//! deterministic completions supply a tight steering estimate: `rollout` (settle
//! the next card, draining into two patience piles) and `rollout_merge` (dump the
//! buffers, then Hu-Tucker). `completion` is the cheaper of the two. Greedy
//! *stepping* drifts; path-kick `iterated_local_search` makes small gains.

use std::time::Instant;

use crate::heuristics::h0;
use crate::machine::{base_len, Card, Move, State};
use crate::sorters::hutucker_sort;
use crate::util::{fxset, Rng};

struct Roller {
    d: Vec<Card>,
    a: Vec<Card>,
    b: Vec<Card>,
    moves: Vec<Move>,
    total: usize,
}

impl Roller {
    fn new(s: &State) -> Self {
        Roller {
            d: s.d.clone(),
            a: s.a.clone(),
            b: s.b.clone(),
            moves: Vec::new(),
            total: s.size(),
        }
    }
    fn sa(&mut self) {
        let c = self.d.pop().unwrap();
        self.a.push(c);
        self.moves.push(Move::SA);
    }
    fn sb(&mut self) {
        let c = self.d.pop().unwrap();
        self.b.push(c);
        self.moves.push(Move::SB);
    }
    fn ma(&mut self) {
        let c = self.a.pop().unwrap();
        self.d.push(c);
        self.moves.push(Move::MA);
    }
    fn mb(&mut self) {
        let c = self.b.pop().unwrap();
        self.d.push(c);
        self.moves.push(Move::MB);
    }
    fn pour_back(&mut self) {
        loop {
            let nxt = base_len(&self.d) + 1;
            if nxt > self.total {
                break;
            }
            let nxt = nxt as Card;
            if self.a.last() == Some(&nxt) {
                self.ma();
            } else if self.b.last() == Some(&nxt) {
                self.mb();
            } else {
                break;
            }
        }
    }
    /// Pour D's top onto a buffer by the 2-pile patience rule (bounce on bury).
    fn route_top(&mut self) {
        let c = *self.d.last().unwrap();
        let ta = self.a.last().copied();
        let tb = self.b.last().copied();
        let a_ok = ta.map_or(false, |t| t > c);
        let b_ok = tb.map_or(false, |t| t > c);
        if a_ok && b_ok {
            if ta.unwrap() <= tb.unwrap() {
                self.sa();
            } else {
                self.sb();
            }
        } else if a_ok {
            self.sa();
        } else if b_ok {
            self.sb();
        } else if self.a.is_empty() {
            self.sa();
        } else if self.b.is_empty() {
            self.sb();
        } else if ta.unwrap() >= tb.unwrap() {
            self.sa();
        } else {
            self.sb();
        }
    }
    fn run(&mut self) {
        self.pour_back();
        while base_len(&self.d) < self.total {
            let k = base_len(&self.d);
            let target = (k + 1) as Card;
            while self.d.len() > k {
                self.route_top();
            }
            if self.a.contains(&target) {
                while *self.a.last().unwrap() != target {
                    self.ma(); // blocker A -> D
                    self.sb(); // D -> B
                }
                self.ma(); // settle target
            } else {
                while *self.b.last().unwrap() != target {
                    self.mb();
                    self.sa();
                }
                self.mb();
            }
            self.pour_back();
        }
    }
}

/// Deterministic settle-next-card completion; works from any state.
pub fn rollout(s: &State) -> Vec<Move> {
    let mut r = Roller::new(s);
    r.run();
    r.moves
}

/// The mirror completion: dump the buffers, then Hu-Tucker.
pub fn rollout_merge(s: &State) -> Vec<Move> {
    let mut d = s.d.clone();
    let mut moves = Vec::new();
    for &c in s.a.iter().rev() {
        d.push(c);
        moves.push(Move::MA);
    }
    for &c in s.b.iter().rev() {
        d.push(c);
        moves.push(Move::MB);
    }
    moves.extend(hutucker_sort(&d));
    moves
}

pub fn rollout_cost(s: &State) -> usize {
    rollout(s).len()
}

/// Forced bounces of the settle-cascade: `(rollout_cost - h0)/2`.
pub fn cascade_bounces(s: &State) -> usize {
    (rollout_cost(s) - h0(s) as usize) / 2
}

/// Inadmissible "cascading charge" `h0 + 2*cascade_bounces` (== settle rollout
/// length). Captures bounce sequencing but overshoots opt — see docs/NOTES.md.
pub fn cascade_charge(s: &State) -> usize {
    rollout_cost(s)
}

/// The cheaper of the two completions (the tight inadmissible estimate).
pub fn completion(s: &State) -> (Vec<Move>, usize) {
    let a = rollout(s);
    let b = rollout_merge(s);
    if a.len() <= b.len() {
        let n = a.len();
        (a, n)
    } else {
        let n = b.len();
        (b, n)
    }
}

pub fn completion_cost(s: &State) -> usize {
    rollout_cost(s).min(rollout_merge(s).len())
}

/// One local-search descent steered by `estimate` (defaults to `completion_cost`),
/// finishing via the strong completion so the result always sorts `start`.
pub fn greedy_solution(
    start: &State,
    rng: &mut Rng,
    epsilon: f64,
    estimate: &dyn Fn(&State) -> usize,
) -> (Vec<Move>, usize) {
    let goal = State::goal(start.size());
    let step_cap = 8 * start.size();
    let mut s = start.clone();
    let mut moves: Vec<Move> = Vec::new();
    let mut visited = fxset();
    visited.insert(start.clone());
    let mut steps = 0;
    while s != goal && steps < step_cap {
        let mut cands: Vec<(usize, State, Move)> = Vec::new();
        for (t, mv) in s.successors() {
            if visited.contains(&t) {
                continue;
            }
            cands.push((estimate(&t), t, mv));
        }
        if cands.is_empty() {
            break;
        }
        cands.sort_by_key(|x| x.0); // stable: ties keep successor order
        let pick = if epsilon > 0.0 && cands.len() > 1 && rng.f64() < epsilon {
            1
        } else {
            0
        };
        let (_, ns, mv) = cands.swap_remove(pick);
        s = ns;
        moves.push(mv);
        visited.insert(s.clone());
        steps += 1;
    }
    moves.extend(completion(&s).0);
    let n = moves.len();
    (moves, n)
}

/// Anytime result of a search.
pub struct Anytime {
    pub best_moves: Vec<Move>,
    pub best_cost: usize,
    pub first_cost: usize,
    pub first_secs: f64,
    pub iters: u64,
}

/// Epsilon-greedy restart search. The first solution is the plain completion;
/// restarts re-descend with perturbed greedy steps. (Greedy stepping drifts, so
/// this rarely beats the first solution -- see `iterated_local_search`.)
pub fn local_search(
    start: &State,
    time_budget: f64,
    seed: u64,
    estimate: Option<&dyn Fn(&State) -> usize>,
) -> Anytime {
    let est: &dyn Fn(&State) -> usize = estimate.unwrap_or(&completion_cost);
    let mut rng = Rng::new(seed);
    let t0 = Instant::now();
    let (first_moves, first_cost) = completion(start);
    let first_secs = t0.elapsed().as_secs_f64();
    let mut best_moves = first_moves;
    let mut best_cost = first_cost;
    let mut iters = 0u64;
    while t0.elapsed().as_secs_f64() < time_budget {
        let eps = if iters == 0 { 0.0 } else { 0.3 };
        let (moves, cost) = greedy_solution(start, &mut rng, eps, est);
        iters += 1;
        if cost < best_cost {
            best_moves = moves;
            best_cost = cost;
        }
    }
    Anytime { best_moves, best_cost, first_cost, first_secs, iters }
}

/// Iterated local search with path kicks: from the best solution, back up to a
/// random state on its path, take a few random legal moves, re-complete; keep the
/// best. Every candidate sorts the given deck.
pub fn iterated_local_search(start: &State, time_budget: f64, seed: u64, max_kick: usize) -> Anytime {
    let goal = State::goal(start.size());
    let mut rng = Rng::new(seed);
    let t0 = Instant::now();
    let (mut best_moves, mut best_cost) = completion(start);
    let first_cost = best_cost;
    let first_secs = t0.elapsed().as_secs_f64();
    let mut iters = 0u64;
    while t0.elapsed().as_secs_f64() < time_budget {
        iters += 1;
        let path = &best_moves;
        let cut = if path.len() > 1 { rng.below(path.len()) } else { 0 };
        let prefix: Vec<Move> = path[..cut].to_vec();
        let mut s = start.applied(&prefix);
        let mut last = prefix.last().copied();
        let mut kick: Vec<Move> = Vec::new();
        let kicks = 1 + rng.below(max_kick);
        for _ in 0..kicks {
            let opts: Vec<(State, Move)> = s
                .successors()
                .into_iter()
                .filter(|(_, mv)| last.map_or(true, |lm| *mv != lm.inv()))
                .collect();
            if opts.is_empty() {
                break;
            }
            let (t, mv) = opts[rng.below(opts.len())].clone();
            s = t;
            last = Some(mv);
            kick.push(mv);
            if s == goal {
                break;
            }
        }
        let mut cand = prefix;
        cand.extend(kick);
        if s != goal {
            cand.extend(completion(&s).0);
        }
        if cand.len() < best_cost {
            best_cost = cand.len();
            best_moves = cand;
        }
    }
    Anytime { best_moves, best_cost, first_cost, first_secs, iters }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rand_state(n: usize, rng: &mut Rng) -> State {
        let p = rng.perm(n);
        let a = rng.below(n + 1);
        let b = rng.below(n - a + 1);
        State {
            d: p[a + b..].to_vec(),
            a: p[..a].to_vec(),
            b: p[a..a + b].to_vec(),
        }
    }

    #[test]
    fn rollouts_sort_arbitrary_states() {
        let mut rng = Rng::new(0);
        for n in 2..=12 {
            for _ in 0..500 {
                let s = rand_state(n, &mut rng);
                let goal = State::goal(n);
                assert_eq!(s.applied(&rollout(&s)), goal);
                assert_eq!(s.applied(&rollout_merge(&s)), goal);
            }
        }
    }

    #[test]
    fn settle_rollout_exact_on_reversal() {
        for n in [8usize, 12, 20, 52] {
            assert_eq!(rollout(&State::reversed_deck(n)).len(), 4 * (n - 1));
            assert_eq!(cascade_bounces(&State::reversed_deck(n)), n - 2);
        }
    }

    #[test]
    fn searches_sort_and_are_anytime() {
        let mut rng = Rng::new(1);
        for n in [6usize, 8, 10] {
            for _ in 0..3 {
                let st = State::from_deck(rng.perm(n));
                let goal = State::goal(n);
                let r = iterated_local_search(&st, 0.1, 2, 6);
                assert_eq!(st.applied(&r.best_moves), goal);
                assert!(r.best_cost <= r.first_cost);
                let ls = local_search(&st, 0.1, 2, None);
                assert_eq!(st.applied(&ls.best_moves), goal);
            }
        }
    }
}
