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
//! batches defends against scheduler noise. the walk is pure syscalls so the
//! numbers swing a lot between machines -- a firecracker vm pays ~4us per
//! syscall, bare metal well under 1us -- so budgets sit ~20x above the slow
//! case and still trip on a real regression like reading every attribute
//! twice or going quadratic on holders
//!
//! ## running
//!
//! ```sh
//! cargo test --test perf_budgets
//! # release for realistic numbers, and --nocapture to see them
//! cargo test --release --test perf_budgets -- --nocapture
//! ```

mod common;

use std::time::{Duration, Instant};

use blockdev::get_devices_at;
use common::FakeRoot;

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

// observed on a firecracker vm in release: 16 disks ~1ms, 256 disks ~15ms.
// on a laptop expect roughly a fifth of that

#[test]
fn walk_16_disks_under_50ms() {
    let r = FakeRoot::new("budget16");
    r.many_disks(16);
    let t = measure(50, || {
        std::hint::black_box(get_devices_at(r.path()).unwrap());
    });
    assert_under("walk 16 disks", t, Duration::from_millis(50));
}

#[test]
fn walk_256_disks_under_500ms() {
    let r = FakeRoot::new("budget256");
    r.many_disks(256);
    let t = measure(5, || {
        std::hint::black_box(get_devices_at(r.path()).unwrap());
    });
    assert_under("walk 256 disks", t, Duration::from_millis(500));
}

#[test]
fn system_filter_256_under_200us() {
    let r = FakeRoot::new("budgetsys");
    r.many_disks(256);
    let parsed = get_devices_at(r.path()).unwrap();
    let t = measure(1000, || {
        std::hint::black_box(std::hint::black_box(&parsed).system());
    });
    assert_under("system() 256 disks", t, Duration::from_micros(200));
}

#[test]
fn find_anywhere_miss_256_under_200us() {
    let r = FakeRoot::new("budgetfind");
    r.many_disks(256);
    let parsed = get_devices_at(r.path()).unwrap();
    let t = measure(1000, || {
        std::hint::black_box(std::hint::black_box(&parsed).find_anywhere("zzz"));
    });
    assert_under(
        "find_anywhere miss 256 disks",
        t,
        Duration::from_micros(200),
    );
}
