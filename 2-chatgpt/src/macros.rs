//! Reusable move macros built exclusively from primitive machine moves.

use crate::{Machine, MachineError, Move, StackId};

fn primitive(source: StackId, destination: StackId) -> Option<Move> {
    match (source, destination) {
        (StackId::A, StackId::D) => Some(Move::AtoD),
        (StackId::D, StackId::A) => Some(Move::DtoA),
        (StackId::D, StackId::B) => Some(Move::DtoB),
        (StackId::B, StackId::D) => Some(Move::BtoD),
        _ => None,
    }
}

/// Moves `count` top cards between stacks.
///
/// Endpoint-to-endpoint transfers are expanded through D and therefore cost
/// two primitive moves per card.
pub fn move_cards(
    machine: &mut Machine,
    count: usize,
    source: StackId,
    destination: StackId,
) -> Result<(), MachineError> {
    if source == destination {
        return Ok(());
    }
    if let Some(movement) = primitive(source, destination) {
        for _ in 0..count {
            machine.apply(movement)?;
        }
    } else {
        let first = primitive(source, StackId::D).expect("source must be an endpoint");
        let second = primitive(StackId::D, destination).expect("destination must be an endpoint");
        for _ in 0..count {
            machine.apply(first)?;
            machine.apply(second)?;
        }
    }
    Ok(())
}

/// Reverses all of D in place in the optimal `4n - 4` moves for `n >= 2`.
///
/// Both endpoint stacks must initially be empty.
pub fn reverse_d(machine: &mut Machine) -> Result<(), MachineError> {
    let n = machine.state().d.len();
    assert!(machine.state().a.is_empty() && machine.state().b.is_empty());
    if n <= 1 {
        return Ok(());
    }
    move_cards(machine, n - 1, StackId::D, StackId::A)?;
    machine.apply(Move::DtoB)?;
    for _ in 0..n - 2 {
        machine.apply(Move::AtoD)?;
        machine.apply(Move::DtoB)?;
    }
    machine.apply(Move::AtoD)?;
    move_cards(machine, n - 1, StackId::B, StackId::D)
}

/// Reverses the top `count` cards from D onto an endpoint in `3k - 2` moves.
pub fn reverse_d_to_endpoint(
    machine: &mut Machine,
    count: usize,
    destination: StackId,
) -> Result<(), MachineError> {
    assert!(matches!(destination, StackId::A | StackId::B));
    if count == 0 {
        return Ok(());
    }
    let temporary = if destination == StackId::A {
        StackId::B
    } else {
        StackId::A
    };
    move_cards(machine, count - 1, StackId::D, temporary)?;
    move_cards(machine, 1, StackId::D, destination)?;
    for _ in 0..count - 1 {
        move_cards(machine, 1, temporary, StackId::D)?;
        move_cards(machine, 1, StackId::D, destination)?;
    }
    Ok(())
}

/// Reverses the top `count` cards from an endpoint onto D in `3k - 2` moves.
pub fn reverse_endpoint_to_d(
    machine: &mut Machine,
    count: usize,
    source: StackId,
) -> Result<(), MachineError> {
    assert!(matches!(source, StackId::A | StackId::B));
    if count == 0 {
        return Ok(());
    }
    let temporary = if source == StackId::A {
        StackId::B
    } else {
        StackId::A
    };
    for _ in 0..count - 1 {
        move_cards(machine, 1, source, StackId::D)?;
        move_cards(machine, 1, StackId::D, temporary)?;
    }
    move_cards(machine, 1, source, StackId::D)?;
    move_cards(machine, count - 1, temporary, StackId::D)
}

/// Reverses `count` cards from one endpoint onto the other in `2k` moves.
pub fn reverse_endpoint_to_endpoint(
    machine: &mut Machine,
    count: usize,
    source: StackId,
    destination: StackId,
) -> Result<(), MachineError> {
    assert!(source != destination);
    assert!(source != StackId::D && destination != StackId::D);
    move_cards(machine, count, source, destination)
}

/// Reverses the top `count` cards in place on an endpoint in `4k - 2` moves.
pub fn reverse_endpoint_in_place(
    machine: &mut Machine,
    count: usize,
    endpoint: StackId,
) -> Result<(), MachineError> {
    assert!(endpoint != StackId::D);
    if count == 0 {
        return Ok(());
    }
    let other = if endpoint == StackId::A {
        StackId::B
    } else {
        StackId::A
    };
    move_cards(machine, count, endpoint, StackId::D)?;
    move_cards(machine, count - 1, StackId::D, other)?;
    move_cards(machine, 1, StackId::D, endpoint)?;
    for _ in 0..count - 1 {
        move_cards(machine, 1, other, StackId::D)?;
        move_cards(machine, 1, StackId::D, endpoint)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::State;

    #[test]
    fn whole_d_regression() {
        let mut machine = Machine::new(&(1..=52).collect::<Vec<_>>()).unwrap();
        reverse_d(&mut machine).unwrap();
        assert_eq!(machine.plan().len(), 204);
        assert_eq!(machine.state().d, (1..=52).rev().collect::<Vec<_>>());
    }

    #[test]
    fn segment_macro_costs_and_protection() {
        for k in 1..=6 {
            let n = k + 2;
            let state = State::new(Vec::new(), (1..=n).collect(), Vec::new()).unwrap();
            let mut machine = Machine::from_state(state);
            reverse_d_to_endpoint(&mut machine, k, StackId::A).unwrap();
            assert_eq!(machine.plan().len(), 3 * k - 2);
            assert_eq!(machine.state().d, vec![k + 1, k + 2]);
            assert_eq!(machine.state().a, (1..=k).collect::<Vec<_>>());

            let before = machine.plan().len();
            reverse_endpoint_to_d(&mut machine, k, StackId::A).unwrap();
            assert_eq!(machine.plan().len() - before, 3 * k - 2);
            assert_eq!(machine.state().d[..k], (1..=k).collect::<Vec<_>>());
        }
    }

    #[test]
    fn endpoint_in_place_regression() {
        for k in 1..=6 {
            let mut machine = Machine::from_state(
                State::new((1..=k).collect(), vec![k + 1], Vec::new()).unwrap(),
            );
            reverse_endpoint_in_place(&mut machine, k, StackId::A).unwrap();
            assert_eq!(machine.plan().len(), 4 * k - 2);
            assert_eq!(machine.state().a, (1..=k).rev().collect::<Vec<_>>());
            assert_eq!(machine.state().d, vec![k + 1]);
        }
    }
}
