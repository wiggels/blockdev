//! wall clock perf budgets -- committed and shipped with the crate
//!
//! criterion baselines live in target/criterion and are gitignored, so a
//! contributor running cargo bench only sees regressions vs their own prior
//! runs. these are the portable backstop: they run under plain cargo test on
//! any machine and catch catastrophic regressions -- think 10x+ -- without
//! any ci infra
//!
//! ## design
//!
//! each test does N iters and takes the best of three batch averages. min of
//! batches defends against scheduler noise -- one preemption can inflate a
//! batch by orders of magnitude when the per call cost is microseconds but
//! it basically never hits all three. budgets are set ~20-50x above what a
//! 2024 commodity laptop measures in release so:
//!
//! * slow ci runners and debug builds do not false positive
//! * a real algorithmic regression -- say size parsing going back through
//!   `serde_json::Value`, or `is_system` going quadratic -- does trip them
//!
//! treat these as a "did we just shoot ourselves in the foot" signal, not a
//! precision instrument. precise regression detection is the criterion
//! benches in benches/parse.rs plus the ci bench workflow
//!
//! ## running
//!
//! ```sh
//! cargo test --test perf_budgets
//! # release for realistic numbers, and --nocapture to see them
//! cargo test --release --test perf_budgets -- --nocapture
//! ```

use std::fmt::Write as _;
use std::time::{Duration, Instant};

use blockdev::parse_lsblk;

const SMALL_FIXTURE: &str = include_str!("../benches/fixtures/small.json");

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

/// run op iters times, three batches, return the min per call average
fn measure<F: FnMut()>(iters: u32, mut op: F) -> Duration {
    op(); // warmup
    let mut best = Duration::MAX;
    for _ in 0..3 {
        let start = Instant::now();
        for _ in 0..iters {
            op();
        }
        let avg = start.elapsed() / iters;
        if avg < best {
            best = avg;
        }
    }
    best
}

/// assert + print so --nocapture shows the observed number next to the cap
fn assert_under(name: &str, observed: Duration, budget: Duration) {
    eprintln!("{name}: observed {observed:?} (budget {budget:?})");
    assert!(
        observed <= budget,
        "{name} took {observed:?}, over the {budget:?} budget -- \
         either a real regression or the budget needs revisiting"
    );
}

/// same shape as the criterion large fixture -- n disks each w/ one partition,
/// human readable sizes so the string path is exercised
fn large_fixture(n_disks: usize) -> String {
    let mut s = String::from("{\"blockdevices\":[");
    for i in 0..n_disks {
        if i > 0 {
            s.push(',');
        }
        write!(
            s,
            "{{\"name\":\"nvme{i}n1\",\"maj:min\":\"259:{i}\",\"rm\":false,\
             \"size\":\"3.5T\",\"ro\":false,\"type\":\"disk\",\"mountpoints\":[null],\
             \"children\":[\
               {{\"name\":\"nvme{i}n1p1\",\"maj:min\":\"259:{i}\",\
                 \"rm\":false,\"size\":\"447.1G\",\"ro\":false,\"type\":\"part\",\
                 \"mountpoints\":[null]}}\
             ]}}",
        )
        .unwrap();
    }
    s.push_str("]}");
    s
}

// ---------------------------------------------------------------------------
// budgets
//
// observed on a 2024 laptop in release: small ~25us, 256 disks ~280us,
// system() on 256 ~1.7us. debug is roughly 5-10x slower and still fits
// ---------------------------------------------------------------------------

#[test]
fn parse_small_realistic_under_2ms() {
    let t = measure(200, || {
        std::hint::black_box(parse_lsblk(std::hint::black_box(SMALL_FIXTURE)).unwrap());
    });
    assert_under("parse small_realistic", t, Duration::from_millis(2));
}

#[test]
fn parse_256_disks_under_20ms() {
    let json = large_fixture(256);
    let t = measure(50, || {
        std::hint::black_box(parse_lsblk(std::hint::black_box(&json)).unwrap());
    });
    assert_under("parse 256 disks", t, Duration::from_millis(20));
}

#[test]
fn system_filter_256_under_200us() {
    let parsed = parse_lsblk(&large_fixture(256)).unwrap();
    let t = measure(1000, || {
        std::hint::black_box(std::hint::black_box(&parsed).system());
    });
    assert_under("system() 256 disks", t, Duration::from_micros(200));
}

#[test]
fn find_anywhere_miss_256_under_200us() {
    // full tree walk on a miss -- catches descendants() going allocation heavy
    let parsed = parse_lsblk(&large_fixture(256)).unwrap();
    let t = measure(1000, || {
        std::hint::black_box(std::hint::black_box(&parsed).find_anywhere("zzz"));
    });
    assert_under(
        "find_anywhere miss 256 disks",
        t,
        Duration::from_micros(200),
    );
}
