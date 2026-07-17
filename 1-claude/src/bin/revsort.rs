//! Experiment (a): is REVERSAL useful for the adjacent-merge sorter?
//!
//! Cost model (exact, = 2*W): C[i][j] = min( leaf-cost(i,j), min_k C[i][k]+C[k+1][j] + (j-i+1) ).
//! Total moves = 2*C[0][n-1].  A block is a LEAF iff monotone:
//!   - ASCENDING block: leaf cost 0 (parks min-on-top in one pour — the baseline's runs).
//!   - DESCENDING block of size s: leaf cost `charge*s` (the realizable reversal tax; a
//!     min-first run cannot park min-on-top by parity, so reorienting it costs extra pours).
//! Sweep `charge`:  charge=0 is the OPTIMISTIC lower bound (reversal free);
//! charge=large reproduces the ascending-only baseline (= Hu-Tucker). Realizable cost lies
//! between, with charge≈2 the comb-style re-stash estimate (~+2 pours/card to reorient).
//!   gap(charge) = baseline - cost(charge);  gap(0)=headroom, gap(2)≈realizable.

use splitmerge::sorters::{ascending_runs, hutucker_cost};
use splitmerge::util::Rng;

struct Leaves {
    n: usize,
    asc: Vec<bool>,  // asc[i*n+j]: deck[i..=j] strictly increasing
    desc: Vec<bool>, // strictly decreasing
}
impl Leaves {
    fn new(deck: &[u8]) -> Self {
        let n = deck.len();
        let mut asc = vec![false; n * n];
        let mut desc = vec![false; n * n];
        for i in 0..n {
            asc[i * n + i] = true;
            desc[i * n + i] = true;
            for j in (i + 1)..n {
                asc[i * n + j] = asc[i * n + (j - 1)] && deck[j - 1] < deck[j];
                desc[i * n + j] = desc[i * n + (j - 1)] && deck[j - 1] > deck[j];
            }
        }
        Leaves { n, asc, desc }
    }
}

/// Optimal `2*C` with descending leaves charged `charge` per card.
fn opt_cost(lv: &Leaves, charge: usize) -> usize {
    let n = lv.n;
    let mut c = vec![vec![0usize; n]; n];
    for len in 2..=n {
        for i in 0..=(n - len) {
            let j = i + len - 1;
            // leaf option
            let mut best = if lv.asc[i * n + j] {
                0
            } else if lv.desc[i * n + j] {
                charge * len
            } else {
                usize::MAX
            };
            // split option
            for k in i..j {
                let v = c[i][k].saturating_add(c[k + 1][j]).saturating_add(len);
                if v < best {
                    best = v;
                }
            }
            c[i][j] = best;
        }
    }
    2 * c[0][n - 1]
}

fn report(label: &str, deck: &[u8]) {
    let lv = Leaves::new(deck);
    let base = opt_cost(&lv, usize::MAX / 4); // charge huge -> ascending-only baseline
    let c: Vec<usize> = (0..=3).map(|ch| opt_cost(&lv, ch)).collect();
    let runs = ascending_runs(deck).len();
    println!(
        "{label:<24} baseline={base:>4} | charge0(free)={:>4} charge1={:>4} charge2={:>4} charge3={:>4} | asc_runs={runs}",
        c[0], c[1], c[2], c[3]
    );
}

fn main() {
    let n = 52usize;

    // sanity: huge-charge DP must equal hutucker_cost (the committed baseline).
    {
        let mut rng = Rng::new(1);
        let mut maxdiff = 0i64;
        for _ in 0..200 {
            let p = rng.perm(n);
            let pd: Vec<u8> = p.iter().map(|&x| x as u8).collect();
            let a = opt_cost(&Leaves::new(&pd), usize::MAX / 4) as i64;
            maxdiff = maxdiff.max((a - hutucker_cost(&p) as i64).abs());
        }
        println!("sanity: max |baseline_DP - hutucker_cost| over 200 random n=52 = {maxdiff} (want 0)\n");
    }

    let reversed: Vec<u8> = (1..=n as u8).rev().collect();
    report("reversed", &reversed);
    let k = n / 2;
    let mut inter: Vec<u8> = Vec::new();
    for t in 0..k {
        inter.push(1 + t as u8);
        inter.push(1 + (k + t) as u8);
    }
    report("interleave(obvious)", &inter);
    let mut blocky: Vec<u8> = (1..=n as u8).collect();
    blocky[4..14].reverse();
    blocky[30..45].reverse();
    report("ascending+2desc-blocks", &blocky);

    // random averages at each charge
    println!();
    let trials = 3000usize;
    let mut rng = Rng::new(2025);
    let charges = [usize::MAX / 4, 0, 1, 2, 3];
    let mut sums = [0u64; 5];
    let mut anygap = 0usize;
    for _ in 0..trials {
        let p = rng.perm(n);
        let pd: Vec<u8> = p.iter().map(|&x| x as u8).collect();
        let lv = Leaves::new(&pd);
        let vals: Vec<usize> = charges.iter().map(|&ch| opt_cost(&lv, ch)).collect();
        for (s, &v) in sums.iter_mut().zip(&vals) {
            *s += v as u64;
        }
        if vals[3] < vals[0] {
            anygap += 1; // charge2 beats baseline
        }
    }
    let mean = |x: u64| x as f64 / trials as f64;
    println!("random n=52 ({trials} decks), mean 2W by descending-leaf charge:");
    println!(
        "  baseline={:.1}  charge0(free,LB)={:.1}  charge1={:.1}  charge2(~realizable)={:.1}  charge3={:.1}",
        mean(sums[0]), mean(sums[1]), mean(sums[2]), mean(sums[3]), mean(sums[4])
    );
    println!(
        "  gap vs baseline:  free={:.1} ({:.1}%)   charge2={:.1} ({:.1}%)   decks where charge2<baseline: {anygap}/{trials}",
        mean(sums[0]) - mean(sums[1]), 100.0 * (mean(sums[0]) - mean(sums[1])) / mean(sums[0]),
        mean(sums[0]) - mean(sums[3]), 100.0 * (mean(sums[0]) - mean(sums[3])) / mean(sums[0]),
    );
}
