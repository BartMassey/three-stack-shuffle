//! The three-stack split-merge machine: deck `D` (hub + I/O) and two LIFO buffers
//! `A`, `B`, with the top of each stack at the end of its vector. Four moves form
//! two involutive inverse pairs (`SA`↔`MA`, `SB`↔`MB`); the configuration graph is
//! undirected. Goal: `D = (1..=n)`, buffers empty. Cost is the number of moves.

/// A card value (`1..=n`, with `n <= 52` the headline target — `u8` suffices).
pub type Card = u8;

/// A machine state `(D, A, B)`, each stack with its top at the end.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct State {
    pub d: Vec<Card>,
    pub a: Vec<Card>,
    pub b: Vec<Card>,
}

/// The four moves. `SA`/`SB` push the deck top onto a buffer; `MA`/`MB` pour a
/// buffer top back onto the deck.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Move {
    SA,
    SB,
    MA,
    MB,
}

impl Move {
    /// The inverse move (`SA`↔`MA`, `SB`↔`MB`).
    #[inline]
    pub fn inv(self) -> Move {
        match self {
            Move::SA => Move::MA,
            Move::MA => Move::SA,
            Move::SB => Move::MB,
            Move::MB => Move::SB,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Move::SA => "SA",
            Move::SB => "SB",
            Move::MA => "MA",
            Move::MB => "MB",
        }
    }
}

/// Length of the committed sorted base: largest `k` with `d[..k] == (1..=k)`.
#[inline]
pub fn base_len(d: &[Card]) -> usize {
    let mut k = 0;
    while k < d.len() && d[k] as usize == k + 1 {
        k += 1;
    }
    k
}

impl State {
    /// The sorted goal state for `n` cards.
    pub fn goal(n: usize) -> State {
        State {
            d: (1..=n as Card).collect(),
            a: Vec::new(),
            b: Vec::new(),
        }
    }

    /// The reversed deck `(n, n-1, ..., 1)` — the hardest known start.
    pub fn reversed_deck(n: usize) -> State {
        State {
            d: (1..=n as Card).rev().collect(),
            a: Vec::new(),
            b: Vec::new(),
        }
    }

    /// A clean start deck from a permutation (buffers empty).
    pub fn from_deck(deck: Vec<Card>) -> State {
        State {
            d: deck,
            a: Vec::new(),
            b: Vec::new(),
        }
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.d.len() + self.a.len() + self.b.len()
    }

    #[inline]
    pub fn base(&self) -> usize {
        base_len(&self.d)
    }

    #[inline]
    pub fn is_goal(&self) -> bool {
        self.a.is_empty() && self.b.is_empty() && base_len(&self.d) == self.d.len()
    }

    /// Apply one move in place (assumes it is legal — the source is non-empty).
    #[inline]
    pub fn apply(&mut self, mv: Move) {
        match mv {
            Move::SA => {
                let c = self.d.pop().unwrap();
                self.a.push(c);
            }
            Move::SB => {
                let c = self.d.pop().unwrap();
                self.b.push(c);
            }
            Move::MA => {
                let c = self.a.pop().unwrap();
                self.d.push(c);
            }
            Move::MB => {
                let c = self.b.pop().unwrap();
                self.d.push(c);
            }
        }
    }

    /// Apply a sequence of moves to a clone, returning the result.
    pub fn applied(&self, moves: &[Move]) -> State {
        let mut s = self.clone();
        for &mv in moves {
            s.apply(mv);
        }
        s
    }

    /// Successor `(state, move)` pairs, in the order `SA, SB, MA, MB`.
    pub fn successors(&self) -> Vec<(State, Move)> {
        let mut out = Vec::with_capacity(4);
        if !self.d.is_empty() {
            let mut sa = self.clone();
            sa.apply(Move::SA);
            out.push((sa, Move::SA));
            let mut sb = self.clone();
            sb.apply(Move::SB);
            out.push((sb, Move::SB));
        }
        if !self.a.is_empty() {
            let mut ma = self.clone();
            ma.apply(Move::MA);
            out.push((ma, Move::MA));
        }
        if !self.b.is_empty() {
            let mut mb = self.clone();
            mb.apply(Move::MB);
            out.push((mb, Move::MB));
        }
        out
    }
}

/// An explicit solution sorting the reversed deck in exactly `4(n-1)` moves — the
/// constructive witness for `opt(reversed deck) <= 4(n-1)` (docs/NOTES.md §I.5).
pub fn comb_solution(n: usize) -> Vec<Move> {
    let mut s = Vec::new();
    if n < 2 {
        return s;
    }
    for _ in 1..n {
        s.push(Move::SB); // phase 1: cards 1..n-1 -> B
    }
    s.push(Move::SA); // phase 2: card n -> A
    for j in (1..=n - 1).rev() {
        // phase 3: pour B back, re-stash on A
        s.push(Move::MB);
        if j >= 2 {
            s.push(Move::SA);
        }
    }
    for _ in 2..=n {
        s.push(Move::MA); // phase 4: pour A
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn goal_and_base() {
        assert_eq!(State::goal(4).d, vec![1, 2, 3, 4]);
        assert_eq!(base_len(&[1, 2, 3, 6, 5]), 3);
        assert_eq!(base_len(&[2, 1, 3]), 0);
        assert_eq!(base_len(&[]), 0);
    }

    #[test]
    fn moves_are_involutive_inverses() {
        let states = [
            State::from_deck(vec![4, 3, 2, 1]),
            State { d: vec![3], a: vec![1], b: vec![2] },
            State { d: vec![], a: vec![2, 4], b: vec![1, 3, 5] },
            State { d: vec![6, 2, 5], a: vec![4, 1], b: vec![3] },
        ];
        for s in &states {
            for (t, mv) in s.successors() {
                // from t, the inverse move must be available and return to s
                let back: Vec<_> = t.successors();
                let found = back.iter().find(|(_, m)| *m == mv.inv());
                assert!(found.is_some(), "{:?} has no inverse from {:?}", mv, t);
                assert_eq!(&found.unwrap().0, s);
            }
        }
    }

    #[test]
    fn comb_sorts_reversed() {
        for n in 2..=20 {
            let moves = comb_solution(n);
            assert_eq!(moves.len(), 4 * (n - 1));
            let end = State::reversed_deck(n).applied(&moves);
            assert_eq!(end, State::goal(n), "comb_solution({}) failed", n);
        }
    }
}
