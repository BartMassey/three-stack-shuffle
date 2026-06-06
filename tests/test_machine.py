"""Machine invariants: moves are involutive inverse pairs, and the comb
construction realises the 4(n-1) upper bound on the reversed deck."""
from splitmerge.machine import (
    GOAL, succ, neighbors, base_len, apply_moves, comb_solution, reversed_deck, INV,
)


def test_goal_and_base():
    assert GOAL(4) == ((1, 2, 3, 4), (), ())
    assert base_len((1, 2, 3, 6, 5)) == 3
    assert base_len((2, 1, 3)) == 0
    assert base_len(()) == 0


def test_moves_are_involutive_inverses():
    # applying a move then its inverse returns to the same state
    states = [
        ((4, 3, 2, 1), (), ()),
        ((3,), (1,), (2,)),
        ((), (2, 4), (1, 3, 5)),
        ((6, 2, 5), (4, 1), (3,)),
    ]
    for s in states:
        for t, mv in succ(s):
            back = {mv2: t2 for t2, mv2 in succ(t)}   # move-name -> state from t
            assert INV[mv] in back, f"{mv} has no inverse available from {t}"
            assert back[INV[mv]] == s, f"{mv} then {INV[mv]} did not return to {s}"


def test_neighbors_match_succ():
    s = ((6, 2, 5), (4, 1), (3,))
    assert neighbors(s) == [t for t, _ in succ(s)]


def test_comb_sorts_reversed_in_4n_minus_4():
    for n in range(2, 21):
        moves = comb_solution(n)
        assert len(moves) == 4 * (n - 1)
        end = apply_moves(reversed_deck(n), moves)
        assert end == GOAL(n), f"comb_solution({n}) did not sort the reversed deck"
