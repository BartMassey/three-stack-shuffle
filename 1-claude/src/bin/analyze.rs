//! Reverse-engineering the optimal "drain-once" strategy: extract optimal move
//! sequences for random n<=14 decks and characterise their structure, contrasted
//! with the Hu-Tucker merge sorter and the settle rollout.
//!
//! ```text
//! analyze traces [n] [k]     dump k annotated optimal solutions
//! analyze stats  [num]       aggregate features (opt vs merge vs rollout)
//! ```

use splitmerge::heuristics::h_joint;
use splitmerge::machine::{base_len, Move, State};
use splitmerge::planner::rollout;
use splitmerge::search::ida_star_path;
use splitmerge::sorters::hutucker_sort;
use splitmerge::util::Rng;

#[allow(dead_code)] // exploratory: not all fields are printed in every mode
struct Feat {
    moves: usize,
    bounces: usize,
    deck_drains: usize, // times |D| hits 0
    buf_empties: usize, // times |A|+|B| hits 0 (mid-solution)
    peak_buf: f64,      // max(|A|+|B|) / n
    phases: usize,      // maximal monotone S/M runs
    max_entries: usize, // max times any card enters D
}

fn features(start: &State, moves: &[Move]) -> Feat {
    let n = start.size();
    let mut s = start.clone();
    let mut entries = vec![0usize; n + 1];
    let (mut deck_drains, mut buf_empties, mut peak, mut phases) = (0usize, 0usize, 0usize, 0usize);
    let mut prev_d = s.d.len();
    let mut prev_buf = s.a.len() + s.b.len();
    let mut prev_merge: Option<bool> = None;
    for &mv in moves {
        let is_merge = matches!(mv, Move::MA | Move::MB);
        s.apply(mv);
        if is_merge {
            entries[*s.d.last().unwrap() as usize] += 1;
        }
        let d = s.d.len();
        let buf = s.a.len() + s.b.len();
        if d == 0 && prev_d > 0 {
            deck_drains += 1;
        }
        if buf == 0 && prev_buf > 0 {
            buf_empties += 1;
        }
        peak = peak.max(buf);
        if prev_merge != Some(is_merge) {
            phases += 1;
            prev_merge = Some(is_merge);
        }
        prev_d = d;
        prev_buf = buf;
    }
    Feat {
        moves: moves.len(),
        bounces: moves.len() / 2 - n,
        deck_drains,
        buf_empties: buf_empties.saturating_sub(1), // discount the final merge-to-goal
        peak_buf: peak as f64 / n as f64,
        phases,
        max_entries: *entries.iter().max().unwrap(),
    }
}

/// Compact move string with phase boundaries, e.g. "SSSB|MMA|...".
fn move_str(moves: &[Move]) -> String {
    let mut out = String::new();
    let mut prev_merge: Option<bool> = None;
    for &mv in moves {
        let is_merge = matches!(mv, Move::MA | Move::MB);
        if prev_merge.is_some() && prev_merge != Some(is_merge) {
            out.push(' ');
        }
        out.push(match mv {
            Move::SA => 'a',
            Move::SB => 'b',
            Move::MA => 'A',
            Move::MB => 'B',
        });
        prev_merge = Some(is_merge);
    }
    out
}

fn traces(n: usize, k: usize) {
    let mut rng = Rng::new(42);
    let mut shown = 0;
    while shown < k {
        let st = State::from_deck(rng.perm(n));
        let (path, _) = ida_star_path(&st, &h_joint, 20_000_000);
        let Some(path) = path else { continue };
        let f = features(&st, &path);
        println!(
            "\ndeck {:?}  opt={} bounces={} drains={} peak={:.2} phases={}",
            st.d, f.moves, f.bounces, f.deck_drains, f.peak_buf, f.phases
        );
        // annotated: lowercase = split to A/B, uppercase = merge from A/B
        println!("  {}", move_str(&path));
        // base / buffer profile sampled
        let mut s = st.clone();
        let mut prof = String::new();
        for (i, &mv) in path.iter().enumerate() {
            s.apply(mv);
            if i % 1.max(path.len() / 24) == 0 {
                prof += &format!("{},{}/{} ", base_len(&s.d), s.a.len(), s.b.len());
            }
        }
        println!("  base,|A|/|B|: {}", prof);
        shown += 1;
    }
}

fn stats(num: usize) {
    println!("feature means over ~{num} random decks  (opt | hutucker | rollout)");
    println!(
        " n |   opt cost      | deck-drains      | peak buf         | phases           | bounces"
    );
    for n in [10usize, 11, 12, 13, 14] {
        let mut rng = Rng::new(100 + n as u64);
        let mut agg: Vec<[f64; 5]> = vec![[0.0; 5]; 3]; // [opt,merge,roll] x [cost,drains,peak,phases,bounces]
        let mut cnt = 0;
        for _ in 0..num {
            let st = State::from_deck(rng.perm(n));
            let (path, _) = ida_star_path(&st, &h_joint, 20_000_000);
            let Some(path) = path else { continue };
            let sols = [path, hutucker_sort(&st.d), rollout(&st)];
            for (i, sol) in sols.iter().enumerate() {
                let f = features(&st, sol);
                agg[i][0] += f.moves as f64;
                agg[i][1] += f.deck_drains as f64;
                agg[i][2] += f.peak_buf;
                agg[i][3] += f.phases as f64;
                agg[i][4] += f.bounces as f64;
            }
            cnt += 1;
        }
        let m = |i: usize, j: usize| agg[i][j] / cnt as f64;
        println!(
            "{n:2} | {:5.1} {:5.1} {:5.1} | {:4.2} {:4.2} {:4.2} | {:.2} {:.2} {:.2} | {:4.1} {:4.1} {:4.1} | {:4.1} {:4.1} {:4.1}",
            m(0,0),m(1,0),m(2,0), m(0,1),m(1,1),m(2,1), m(0,2),m(1,2),m(2,2), m(0,3),m(1,3),m(2,3), m(0,4),m(1,4),m(2,4)
        );
    }
}

/// Per-card entry counts in optimal solutions: does the max grow with n?
fn bounce_dist(num: usize) {
    println!("optimal per-card entries into D (= 1 + bounces), {num} random decks each");
    println!(" n |  opt  | mean entries | max entries | %cards bounce>=1 | %cards bounce>=2");
    for n in [8usize, 10, 12, 14] {
        let mut rng = Rng::new(300 + n as u64);
        let (mut sopt, mut smean, mut smax, mut sb1, mut sb2, mut cnt) =
            (0.0, 0.0, 0.0, 0.0, 0.0, 0usize);
        for _ in 0..num {
            let st = State::from_deck(rng.perm(n));
            let (path, _) = ida_star_path(&st, &h_joint, 20_000_000);
            let Some(path) = path else { continue };
            let mut entries = vec![0usize; n + 1];
            let mut s = st.clone();
            for &mv in &path {
                s.apply(mv);
                if matches!(mv, Move::MA | Move::MB) {
                    entries[*s.d.last().unwrap() as usize] += 1;
                }
            }
            let nz: Vec<usize> = entries.into_iter().filter(|&e| e > 0).collect();
            sopt += path.len() as f64;
            smean += nz.iter().sum::<usize>() as f64 / nz.len() as f64;
            smax += *nz.iter().max().unwrap() as f64;
            sb1 += nz.iter().filter(|&&e| e >= 2).count() as f64 / nz.len() as f64;
            sb2 += nz.iter().filter(|&&e| e >= 3).count() as f64 / nz.len() as f64;
            cnt += 1;
        }
        let c = cnt as f64;
        println!(
            " {n:2} | {:5.1} |    {:.2}      |    {:.2}     |     {:4.0}%        |     {:4.0}%",
            sopt / c,
            smean / c,
            smax / c,
            100.0 * sb1 / c,
            100.0 * sb2 / c
        );
    }
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let cmd = a.get(1).map(|s| s.as_str()).unwrap_or("stats");
    match cmd {
        "bounces" => bounce_dist(a.get(2).and_then(|s| s.parse().ok()).unwrap_or(60)),
        "traces" => traces(
            a.get(2).and_then(|s| s.parse().ok()).unwrap_or(9),
            a.get(3).and_then(|s| s.parse().ok()).unwrap_or(6),
        ),
        "stats" => stats(a.get(2).and_then(|s| s.parse().ok()).unwrap_or(60)),
        other => eprintln!("unknown: {other}"),
    }
}
