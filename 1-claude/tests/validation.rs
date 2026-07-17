//! Cross-validation against the Python reference's invariants. Run with
//! `cargo test --release` (the exhaustive n=8 sweep is heavy in debug).

use splitmerge::heuristics::{h0, h_best, h_joint, soft_conflict_graph};
use splitmerge::machine::State;
use splitmerge::oct::{exact_oct, oct_pre_bruteforce, DEFAULT_BUDGET};
use splitmerge::search::{bfs_dist, ida_star};
use splitmerge::util::Rng;

#[test]
fn admissibility_and_dominance_exhaustive() {
    // every reachable state: h0 <= h_best <= h_joint <= dist, for n = 6, 7, 8
    for n in [6usize, 7, 8] {
        let dist = bfs_dist(n);
        for (s, &d) in &dist {
            let a = h0(s);
            let b = h_best(s);
            let c = h_joint(s);
            assert!(a <= b && b <= c, "dominance violated at {:?}", s);
            assert!(c <= d, "h_joint inadmissible at {:?}: {} > {}", s, c, d);
        }
    }
}

#[test]
fn oct_matches_oracle_small() {
    for n in [4usize, 5, 6] {
        for (s, _) in bfs_dist(n) {
            let (vals, adj, pre, _) = soft_conflict_graph(&s);
            if !vals.is_empty() {
                let (v, exact) = exact_oct(&vals, &adj, &pre, DEFAULT_BUDGET);
                assert!(exact);
                assert_eq!(v, oct_pre_bruteforce(&vals, &adj, &pre), "OCT at {:?}", s);
            }
        }
    }
}

#[test]
fn ida_matches_bfs() {
    let dist = bfs_dist(7);
    let mut rng = Rng::new(7);
    for _ in 0..12 {
        let st = State::from_deck(rng.perm(7));
        let (c, _) = ida_star(&st, &h_joint, 2_000_000);
        assert_eq!(c.unwrap(), dist[&st]);
    }
}

#[test]
fn ida_reversal_optimum() {
    for n in [8usize, 10, 12] {
        let (c, _) = ida_star(&State::reversed_deck(n), &h_joint, 2_000_000);
        assert_eq!(c.unwrap(), 4 * (n as u32 - 1));
    }
}

#[test]
fn reversed_52_solves_to_204_in_204_nodes() {
    let (c, nodes) = ida_star(&State::reversed_deck(52), &h_joint, 2_000_000);
    assert_eq!(c, Some(204));
    assert_eq!(nodes, 204);
}

#[test]
fn reversal_exact_path_nodes_hbest() {
    // h_best is exact along the optimal reversal path => nodes == cost
    for n in [10usize, 12, 20] {
        let (c, nodes) = ida_star(&State::reversed_deck(n), &h_best, 2_000_000);
        assert_eq!(c, Some(4 * (n as u32 - 1)));
        assert_eq!(nodes, 4 * (n as u64 - 1));
    }
}
