//! Scratch: EXHAUSTIVE n=9 confirmation of the "forced double-transfer" boundary.
//! dbx.rs proved n<=8 has none (exhaustive) and found the first forced n=9 deck by
//! *sampling* (1 in 4000). This enumerates all 9! decks on the real, parking-capable
//! machine to (a) confirm forced doubles really exist at n=9, (b) report the exact
//! count, and (c) name the genuine lex-FIRST smallest forced deck — killing the
//! "maybe there are smaller ones" uncertainty. Threaded across cores.

use splitmerge::heuristics::h_joint;
use splitmerge::machine::{base_len, Card, Move, State};
use splitmerge::search::ida_star_path;
use splitmerge::util::{fxset, FxHashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

fn arrivals(start: &State, path: &[Move]) -> Vec<u32> {
    let n = start.size();
    let mut e = vec![0u32; n + 1];
    let mut s = start.clone();
    for &mv in path {
        s.apply(mv);
        if matches!(mv, Move::MA | Move::MB) {
            e[*s.d.last().unwrap() as usize] += 1;
        }
    }
    e
}

/// Does a length-`opt` solution exist with every card arriving <= 2 times?
/// (Parking is permitted — only the per-card arrival count is capped.)
fn exists_simple_optimal(start: &State, opt: u32) -> bool {
    let goal = State::goal(start.size());
    let mut entries = vec![0u32; start.size() + 1];
    let mut on_path: FxHashSet<State> = fxset();
    on_path.insert(start.clone());
    fn dfs(
        s: &State,
        g: u32,
        bound: u32,
        last: Option<Move>,
        goal: &State,
        entries: &mut [u32],
        on_path: &mut FxHashSet<State>,
    ) -> bool {
        if s == goal {
            return g == bound;
        }
        if g + h_joint(s) > bound {
            return false;
        }
        for (t, mv) in s.successors() {
            if last == Some(mv.inv()) || on_path.contains(&t) {
                continue;
            }
            let mut undo: Option<usize> = None;
            if matches!(mv, Move::MA | Move::MB) {
                let c = *t.d.last().unwrap() as usize;
                if entries[c] >= 2 {
                    continue;
                }
                entries[c] += 1;
                undo = Some(c);
            }
            on_path.insert(t.clone());
            let ok = dfs(&t, g + 1, bound, Some(mv), goal, entries, on_path);
            on_path.remove(&t);
            if let Some(c) = undo {
                entries[c] -= 1;
            }
            if ok {
                return true;
            }
        }
        false
    }
    dfs(start, 0, opt, None, &goal, &mut entries, &mut on_path)
}

/// Unrank the `idx`-th permutation of {1..=n} in lexicographic order (factorial
/// number system), so threads can split the index range and global lex order is
/// preserved without materialising all perms.
fn unrank(idx: usize, n: usize) -> Vec<Card> {
    let mut avail: Vec<Card> = (1..=n as Card).collect();
    let mut fact = vec![1usize; n];
    for i in 1..n {
        fact[i] = fact[i - 1] * i;
    }
    let mut rem = idx;
    let mut out = Vec::with_capacity(n);
    for i in (0..n).rev() {
        let f = fact[i];
        let j = rem / f;
        rem %= f;
        out.push(avail.remove(j));
    }
    out
}

struct Hit {
    deck: Vec<Card>,
    opt: u32,
    doublers: Vec<usize>,
    arr: Vec<u32>,
    reburial: bool, // some doubler landed on a SMALLER card (the documented atom)
}

/// Replay `path`; for each doubler card, was any of its buffer-placements onto a
/// SMALLER card (re-burial), or were they all onto larger/empty (hot-potato)?
/// Returns true if at least one doubler exhibits a re-burial landing.
fn has_reburial(start: &State, path: &[Move], doublers: &[usize]) -> bool {
    let mut s = start.clone();
    let mut found = false;
    for &mv in path {
        // a split moves D's top into a buffer; inspect the landing
        if matches!(mv, Move::SA | Move::SB) {
            let c = *s.d.last().unwrap() as usize;
            if doublers.contains(&c) {
                let dest_top = match mv {
                    Move::SA => s.a.last().copied(),
                    _ => s.b.last().copied(),
                };
                if let Some(t) = dest_top {
                    if (t as usize) < c {
                        found = true; // landed on a smaller unsettled card
                    }
                }
            }
        }
        s.apply(mv);
    }
    found
}

fn main() {
    const N: usize = 9;
    let total: usize = (1..=N).product();
    let nthreads = std::thread::available_parallelism().map(|x| x.get()).unwrap_or(8);
    let next = AtomicUsize::new(0);
    let chunk = 2000usize;
    let occurs = AtomicUsize::new(0);
    let forced_count = AtomicUsize::new(0);
    let forced: Mutex<Vec<Hit>> = Mutex::new(Vec::new());
    let base_free_forced = AtomicUsize::new(0);

    std::thread::scope(|scope| {
        for _ in 0..nthreads {
            scope.spawn(|| loop {
                let start = next.fetch_add(chunk, Ordering::Relaxed);
                if start >= total {
                    break;
                }
                let end = (start + chunk).min(total);
                for idx in start..end {
                    let deck = unrank(idx, N);
                    let st = State::from_deck(deck.clone());
                    let (path, _) = ida_star_path(&st, &h_joint, 20_000_000);
                    let Some(path) = path else { continue };
                    let arr = arrivals(&st, &path);
                    if *arr.iter().max().unwrap() < 3 {
                        continue;
                    }
                    occurs.fetch_add(1, Ordering::Relaxed);
                    let opt = path.len() as u32;
                    if exists_simple_optimal(&st, opt) {
                        continue;
                    }
                    forced_count.fetch_add(1, Ordering::Relaxed);
                    if base_len(&deck) == 0 {
                        base_free_forced.fetch_add(1, Ordering::Relaxed);
                    }
                    let doublers: Vec<usize> = (1..arr.len()).filter(|&c| arr[c] >= 3).collect();
                    let reburial = has_reburial(&st, &path, &doublers);
                    forced.lock().unwrap().push(Hit {
                        deck: deck.clone(),
                        opt,
                        doublers,
                        arr: arr[1..].to_vec(),
                        reburial,
                    });
                }
            });
        }
    });

    let mut forced = forced.into_inner().unwrap();
    forced.sort_by(|a, b| a.deck.cmp(&b.deck));
    let n_reburial = forced.iter().filter(|h| h.reburial).count();
    let n_hotpotato = forced.len() - n_reburial;
    let min_opt = forced.iter().map(|h| h.opt).min().unwrap_or(0);
    let n_min_opt = forced.iter().filter(|h| h.opt == min_opt).count();
    // among the doublers, how often is the smallest card (card 1) a doubler?
    let n_card1 = forced.iter().filter(|h| h.doublers.contains(&1)).count();
    println!("=== EXHAUSTIVE n={N} ({total} decks, {nthreads} threads) ===");
    println!(
        "double OCCURS in optimum: {}   FORCED double-transfer: {}   (all base-free: {})",
        occurs.load(Ordering::Relaxed),
        forced_count.load(Ordering::Relaxed),
        base_free_forced.load(Ordering::Relaxed) == forced.len(),
    );
    println!(
        "mechanism (in the IDA*-returned optimum): re-burial(lands-on-smaller)={n_reburial}  hot-potato(lands-only-on-larger)={n_hotpotato}",
    );
    println!("forced decks whose doubler set includes card 1 (the smallest): {n_card1}");
    println!("min opt among forced decks = {min_opt}  ({n_min_opt} forced decks at that opt)");
    if let Some(h) = forced.first() {
        println!(
            "\nlex-FIRST forced deck: {:?}  opt={}  doubler-cards={:?}  arrivals={:?}  mechanism={}",
            h.deck, h.opt, h.doublers, h.arr,
            if h.reburial { "re-burial" } else { "HOT-POTATO" },
        );
    }
    println!("\nall forced decks at min opt = {min_opt} (lex):");
    for h in forced.iter().filter(|h| h.opt == min_opt) {
        println!(
            "  {:?}  doublers={:?}  {}",
            h.deck, h.doublers,
            if h.reburial { "re-burial" } else { "HOT-POTATO" },
        );
    }
}
