//! Three-stack card-machine simulation, constructive sorting, and exact search.
//!
//! Stack vectors use the specification's **top-to-bottom** order. This makes
//! states, diagnostics, and hash keys unambiguous; the small decks targeted by
//! the project make removing the first vector element an acceptable tradeoff.

#![forbid(unsafe_code)]

pub mod algorithms;
pub mod macros;
pub mod random;
pub mod search;

use std::error::Error;
use std::fmt;

/// One of the three physical stacks in the path `A - D - B`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StackId {
    /// Left endpoint.
    A,
    /// Center stack.
    D,
    /// Right endpoint.
    B,
}

/// A primitive legal move between adjacent stacks.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Move {
    /// Move the top card from A to D.
    AtoD,
    /// Move the top card from D to A.
    DtoA,
    /// Move the top card from D to B.
    DtoB,
    /// Move the top card from B to D.
    BtoD,
}

impl Move {
    /// Returns the source and destination stacks.
    #[must_use]
    pub const fn endpoints(self) -> (StackId, StackId) {
        match self {
            Self::AtoD => (StackId::A, StackId::D),
            Self::DtoA => (StackId::D, StackId::A),
            Self::DtoB => (StackId::D, StackId::B),
            Self::BtoD => (StackId::B, StackId::D),
        }
    }

    /// Returns the inverse primitive move.
    #[must_use]
    pub const fn inverse(self) -> Self {
        match self {
            Self::AtoD => Self::DtoA,
            Self::DtoA => Self::AtoD,
            Self::DtoB => Self::BtoD,
            Self::BtoD => Self::DtoB,
        }
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::AtoD => "A->D",
            Self::DtoA => "D->A",
            Self::DtoB => "D->B",
            Self::BtoD => "B->D",
        };
        f.write_str(text)
    }
}

/// An immutable machine state, with every stack stored top-to-bottom.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct State {
    /// Left endpoint, top first.
    pub a: Vec<usize>,
    /// Center stack, top first.
    pub d: Vec<usize>,
    /// Right endpoint, top first.
    pub b: Vec<usize>,
}

impl State {
    /// Constructs a state and validates the `1..=n` card invariant.
    pub fn new(a: Vec<usize>, d: Vec<usize>, b: Vec<usize>) -> Result<Self, MachineError> {
        let state = Self { a, d, b };
        state.validate()?;
        Ok(state)
    }

    /// Constructs the initial state for a permutation.
    pub fn initial(deck: &[usize]) -> Result<Self, MachineError> {
        Self::new(Vec::new(), deck.to_vec(), Vec::new())
    }

    /// Constructs the goal state for `n` cards.
    #[must_use]
    pub fn goal(n: usize) -> Self {
        Self {
            a: Vec::new(),
            d: (1..=n).collect(),
            b: Vec::new(),
        }
    }

    /// Returns the total card count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.a.len() + self.d.len() + self.b.len()
    }

    /// Returns whether the state contains no cards.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Validates that every label in `1..=n` occurs exactly once.
    pub fn validate(&self) -> Result<(), MachineError> {
        let n = self.len();
        let mut seen = vec![false; n + 1];
        for &card in self.a.iter().chain(&self.d).chain(&self.b) {
            if card == 0 || card > n || seen[card] {
                return Err(MachineError::InvalidCards);
            }
            seen[card] = true;
        }
        Ok(())
    }

    /// Returns legal neighboring states in deterministic move order.
    #[must_use]
    pub fn neighbors(&self) -> Vec<(Self, Move)> {
        [Move::AtoD, Move::DtoA, Move::DtoB, Move::BtoD]
            .into_iter()
            .filter_map(|movement| {
                let mut next = self.clone();
                next.apply(movement).ok()?;
                Some((next, movement))
            })
            .collect()
    }

    fn stack_mut(&mut self, id: StackId) -> &mut Vec<usize> {
        match id {
            StackId::A => &mut self.a,
            StackId::D => &mut self.d,
            StackId::B => &mut self.b,
        }
    }

    fn apply(&mut self, movement: Move) -> Result<(), MachineError> {
        let (source, destination) = movement.endpoints();
        let card = self
            .stack_mut(source)
            .first()
            .copied()
            .ok_or(MachineError::EmptySource(source))?;
        self.stack_mut(source).remove(0);
        self.stack_mut(destination).insert(0, card);
        debug_assert!(self.validate().is_ok());
        Ok(())
    }
}

/// A sequence of primitive moves.
pub type Plan = Vec<Move>;

/// Errors produced by validation or illegal simulation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MachineError {
    /// The cards are not exactly one copy of each label in `1..=n`.
    InvalidCards,
    /// A move was attempted from an empty stack.
    EmptySource(StackId),
    /// A returned plan was legal but did not reach the exact goal.
    NotSorted(State),
}

impl fmt::Display for MachineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCards => f.write_str("cards must be a permutation of 1..=n"),
            Self::EmptySource(stack) => write!(f, "cannot move from empty stack {stack:?}"),
            Self::NotSorted(state) => write!(f, "plan ended in non-goal state {state:?}"),
        }
    }
}

impl Error for MachineError {}

/// Mutable simulator that records every primitive move it executes.
#[derive(Clone, Debug)]
pub struct Machine {
    state: State,
    initial_n: usize,
    plan: Plan,
}

impl Machine {
    /// Creates a simulator from an arbitrary valid state.
    pub fn from_state(state: State) -> Self {
        let initial_n = state.len();
        Self {
            state,
            initial_n,
            plan: Vec::new(),
        }
    }

    /// Creates a simulator in the initial state for `deck`.
    pub fn new(deck: &[usize]) -> Result<Self, MachineError> {
        Ok(Self::from_state(State::initial(deck)?))
    }

    /// Returns the current immutable state.
    #[must_use]
    pub const fn state(&self) -> &State {
        &self.state
    }

    /// Returns the emitted primitive plan.
    #[must_use]
    pub fn plan(&self) -> &[Move] {
        &self.plan
    }

    /// Applies and records exactly one primitive move.
    pub fn apply(&mut self, movement: Move) -> Result<(), MachineError> {
        self.state.apply(movement)?;
        self.plan.push(movement);
        debug_assert_eq!(self.state.len(), self.initial_n);
        Ok(())
    }

    /// Applies a complete plan, stopping at the first illegal move.
    pub fn apply_plan(&mut self, plan: &[Move]) -> Result<(), MachineError> {
        for &movement in plan {
            self.apply(movement)?;
        }
        Ok(())
    }

    /// Returns and clears the recorded plan without changing the state.
    pub fn take_plan(&mut self) -> Plan {
        std::mem::take(&mut self.plan)
    }
}

/// Replays `plan` from an arbitrary state and returns the final state.
pub fn replay(state: &State, plan: &[Move]) -> Result<State, MachineError> {
    let mut machine = Machine::from_state(state.clone());
    machine.apply_plan(plan)?;
    Ok(machine.state)
}

/// Replays a sorting plan and requires the exact goal state.
pub fn validate_sort_plan(deck: &[usize], plan: &[Move]) -> Result<(), MachineError> {
    let final_state = replay(&State::initial(deck)?, plan)?;
    if final_state == State::goal(deck.len()) {
        Ok(())
    } else {
        Err(MachineError::NotSorted(final_state))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primitive_moves_are_recorded_and_reversible() {
        let state = State::initial(&[2, 1]).unwrap();
        for (child, movement) in state.neighbors() {
            let restored = replay(&child, &[movement.inverse()]).unwrap();
            assert_eq!(restored, state);
        }
    }

    #[test]
    fn invalid_cards_are_rejected() {
        assert_eq!(State::initial(&[1, 1]), Err(MachineError::InvalidCards));
        assert_eq!(State::initial(&[0]), Err(MachineError::InvalidCards));
    }
}
