//! Constructive no-reversal merge sorters that emit real machine moves (verified
//! by replay). Three adjacent-merge sorters differing only in the merge tree:
//! `natural` (multi-pass, `2n*ceil(log2 r)`), `top_down` and `hu_tucker`
//! (`2*W(tree)`, `W` minimal for Hu-Tucker). See docs/NOTES.md §I.3.

use crate::machine::{Card, Move};

/// Sizes of the maximal ascending (bottom-to-top) runs of `deck`.
pub fn ascending_runs(deck: &[Card]) -> Vec<usize> {
    if deck.is_empty() {
        return Vec::new();
    }
    let mut sizes = Vec::new();
    let mut cur = 1;
    for i in 1..deck.len() {
        if deck[i] > deck[i - 1] {
            cur += 1;
        } else {
            sizes.push(cur);
            cur = 1;
        }
    }
    sizes.push(cur);
    sizes
}

/// A tiny live machine that records the moves it makes.
struct Emitter {
    d: Vec<Card>,
    a: Vec<Card>,
    b: Vec<Card>,
    moves: Vec<Move>,
}

impl Emitter {
    fn new(deck: &[Card]) -> Self {
        Emitter {
            d: deck.to_vec(),
            a: Vec::new(),
            b: Vec::new(),
            moves: Vec::new(),
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
    /// Merge the top `na` of A with the top `nb` of B back onto D (both runs have
    /// their minimum on top), by exact count.
    fn merge_ab(&mut self, mut na: usize, mut nb: usize) {
        while na > 0 && nb > 0 {
            if *self.a.last().unwrap() < *self.b.last().unwrap() {
                self.ma();
                na -= 1;
            } else {
                self.mb();
                nb -= 1;
            }
        }
        while na > 0 {
            self.ma();
            na -= 1;
        }
        while nb > 0 {
            self.mb();
            nb -= 1;
        }
    }
}

/// Multi-pass natural merge. Cost exactly `2n*ceil(log2 r)`.
pub fn natural_sort(deck: &[Card]) -> Vec<Move> {
    let mut m = Emitter::new(deck);
    let mut runs = ascending_runs(&m.d);
    while runs.len() > 1 {
        let (mut a_runs, mut b_runs) = (Vec::new(), Vec::new());
        let mut to_a = true;
        for &len in runs.iter().rev() {
            if to_a {
                for _ in 0..len {
                    m.sa();
                }
                a_runs.push(len);
            } else {
                for _ in 0..len {
                    m.sb();
                }
                b_runs.push(len);
            }
            to_a = !to_a;
        }
        let mut new_runs = Vec::new();
        while let (Some(la), Some(lb)) = (a_runs.last().copied(), b_runs.last().copied()) {
            a_runs.pop();
            b_runs.pop();
            m.merge_ab(la, lb);
            new_runs.push(la + lb);
        }
        for &len in a_runs.iter().rev() {
            for _ in 0..len {
                m.ma();
            }
            new_runs.push(len);
        }
        for &len in b_runs.iter().rev() {
            for _ in 0..len {
                m.mb();
            }
            new_runs.push(len);
        }
        runs = new_runs;
    }
    m.moves
}

/// A binary merge tree over the runs.
pub enum Tree {
    Leaf(usize),
    Node(Box<Tree>, Box<Tree>, usize),
}

impl Tree {
    pub fn size(&self) -> usize {
        match self {
            Tree::Leaf(s) => *s,
            Tree::Node(_, _, s) => *s,
        }
    }
}

/// `W(T) = sum_i size_i * depth_i` (root at depth 0).
pub fn weighted_path_length(t: &Tree) -> usize {
    fn rec(t: &Tree, depth: usize) -> usize {
        match t {
            Tree::Leaf(s) => s * depth,
            Tree::Node(l, r, _) => rec(l, depth + 1) + rec(r, depth + 1),
        }
    }
    rec(t, 0)
}

fn prefix_sums(run_sizes: &[usize]) -> Vec<usize> {
    let mut pre = vec![0usize; run_sizes.len() + 1];
    for (i, &s) in run_sizes.iter().enumerate() {
        pre[i + 1] = pre[i] + s;
    }
    pre
}

/// Adaptive top-down tree: split at the boundary nearest the card midpoint.
pub fn build_topdown(run_sizes: &[usize]) -> Tree {
    if run_sizes.is_empty() {
        return Tree::Leaf(0);
    }
    let pre = prefix_sums(run_sizes);
    fn build(i: usize, j: usize, run_sizes: &[usize], pre: &[usize]) -> Tree {
        if j - i == 1 {
            return Tree::Leaf(run_sizes[i]);
        }
        let target = (pre[i] + pre[j]) as f64 / 2.0;
        let mut best_k = i + 1;
        let mut best_d = f64::INFINITY;
        for k in (i + 1)..j {
            let d = (pre[k] as f64 - target).abs();
            if d < best_d {
                best_d = d;
                best_k = k;
            }
        }
        let left = build(i, best_k, run_sizes, pre);
        let right = build(best_k, j, run_sizes, pre);
        Tree::Node(Box::new(left), Box::new(right), pre[j] - pre[i])
    }
    build(0, run_sizes.len(), run_sizes, &pre)
}

/// Optimal alphabetic (order-constrained) merge tree via the optimal-BST DP.
pub fn build_hutucker(run_sizes: &[usize]) -> Tree {
    let r = run_sizes.len();
    if r == 0 {
        return Tree::Leaf(0);
    }
    if r == 1 {
        return Tree::Leaf(run_sizes[0]);
    }
    let pre = prefix_sums(run_sizes);
    let mut c = vec![vec![0usize; r]; r];
    let mut kbest = vec![vec![0usize; r]; r];
    for i in 0..r {
        kbest[i][i] = i;
    }
    for length in 2..=r {
        for i in 0..=(r - length) {
            let j = i + length - 1;
            let w = pre[j + 1] - pre[i];
            let mut best = usize::MAX;
            let mut bk = i;
            for k in i..j {
                let v = c[i][k] + c[k + 1][j];
                if v < best {
                    best = v;
                    bk = k;
                }
            }
            c[i][j] = best + w;
            kbest[i][j] = bk;
        }
    }
    fn build(i: usize, j: usize, run_sizes: &[usize], pre: &[usize], kbest: &[Vec<usize>]) -> Tree {
        if i == j {
            return Tree::Leaf(run_sizes[i]);
        }
        let k = kbest[i][j];
        let left = build(i, k, run_sizes, pre, kbest);
        let right = build(k + 1, j, run_sizes, pre, kbest);
        Tree::Node(Box::new(left), Box::new(right), pre[j + 1] - pre[i])
    }
    build(0, r - 1, run_sizes, &pre, &kbest)
}

/// Emit the moves that sort `deck` by realizing `tree` (cost `2*W`).
pub fn realize_tree(deck: &[Card], tree: &Tree) -> Vec<Move> {
    let mut m = Emitter::new(deck);
    fn rec(t: &Tree, m: &mut Emitter) {
        if let Tree::Node(l, r, _) = t {
            rec(r, m); // sort the upper (top-of-D) child
            for _ in 0..r.size() {
                m.sa(); // park it onto A
            }
            rec(l, m);
            for _ in 0..l.size() {
                m.sb(); // park onto B
            }
            m.merge_ab(r.size(), l.size());
        }
    }
    rec(tree, &mut m);
    m.moves
}

pub fn topdown_sort(deck: &[Card]) -> Vec<Move> {
    realize_tree(deck, &build_topdown(&ascending_runs(deck)))
}

pub fn hutucker_sort(deck: &[Card]) -> Vec<Move> {
    realize_tree(deck, &build_hutucker(&ascending_runs(deck)))
}

// ----- closed-form costs (no replay) ------------------------------------------

pub fn natural_cost(deck: &[Card]) -> usize {
    let n = deck.len();
    let r = ascending_runs(deck).len();
    if r <= 1 {
        0
    } else {
        2 * n * (usize::BITS - (r - 1).leading_zeros()) as usize // 2n * ceil(log2 r)
    }
}

pub fn topdown_cost(deck: &[Card]) -> usize {
    2 * weighted_path_length(&build_topdown(&ascending_runs(deck)))
}

pub fn hutucker_cost(deck: &[Card]) -> usize {
    2 * weighted_path_length(&build_hutucker(&ascending_runs(deck)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::State;

    fn perms(n: usize, f: &mut dyn FnMut(&[Card])) {
        let mut p: Vec<Card> = (1..=n as Card).collect();
        permute(&mut p, 0, f);
    }
    fn permute(p: &mut Vec<Card>, i: usize, f: &mut dyn FnMut(&[Card])) {
        if i == p.len() {
            f(p);
            return;
        }
        for j in i..p.len() {
            p.swap(i, j);
            permute(p, i + 1, f);
            p.swap(i, j);
        }
    }

    #[test]
    fn all_perms_sort_and_count() {
        for n in 1..=7 {
            let goal = State::goal(n);
            perms(n, &mut |p| {
                for (sort, cost) in [
                    (natural_sort as fn(&[Card]) -> Vec<Move>, natural_cost as fn(&[Card]) -> usize),
                    (topdown_sort, topdown_cost),
                    (hutucker_sort, hutucker_cost),
                ] {
                    let moves = sort(p);
                    assert_eq!(State::from_deck(p.to_vec()).applied(&moves), goal);
                    assert_eq!(moves.len(), cost(p));
                }
            });
        }
    }

    #[test]
    fn worst_case_reversed_52() {
        let rev: Vec<Card> = (1..=52).rev().collect();
        assert_eq!(natural_cost(&rev), 624);
        assert_eq!(topdown_cost(&rev), 600);
        assert_eq!(hutucker_cost(&rev), 600);
        assert_eq!(
            State::from_deck(rev.clone()).applied(&hutucker_sort(&rev)),
            State::goal(52)
        );
    }
}
