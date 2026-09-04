//! criterion benches -- regression detection, not micro-opt
//!
//! two groups:
//!
//! * `walk` -- `get_devices_at` over fake sysroots of fixed size. this is the
//!   deterministic number ci diffs against gh-pages history, since a real
//!   machine's layout varies run to run
//! * `live` -- `get_devices()` on the actual host, plus `lsblk --json --bytes`
//!   spawned for reference so the pages graph keeps showing what the walk
//!   saves. skipped where there is no /sys or no lsblk
//! * `calib` -- fixed work that no code change touches. github runners come
//!   in ~2x different speed classes so raw ns across runs is mostly which
//!   machine you got. ci divides every bench by the matching calibration
//!   from the same run and publishes that ratio next to the raw numbers
//!
//! ```sh
//! cargo bench --bench devices -- --save-baseline main
//! # ... make changes ...
//! cargo bench --bench devices -- --baseline main
//! ```

#[path = "../tests/common/mod.rs"]
mod common;

use std::hint::black_box;
use std::process::Command;
use std::time::Duration;

use blockdev::{get_devices, get_devices_at};
use common::FakeRoot;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

fn bench_walk(c: &mut Criterion) {
    let mut group = c.benchmark_group("walk");
    for n in [16u32, 256] {
        let r = FakeRoot::new(&format!("bench{n}"));
        r.many_disks(n);
        group.bench_with_input(BenchmarkId::new("disks", n), &n, |b, _| {
            b.iter(|| get_devices_at(black_box(r.path())).unwrap());
        });
    }
    group.finish();
}

fn bench_filters(c: &mut Criterion) {
    let r = FakeRoot::new("benchfilters");
    r.many_disks(256);
    let parsed = get_devices_at(r.path()).unwrap();

    let mut group = c.benchmark_group("filters");
    group.bench_function("system_256", |b| b.iter(|| black_box(&parsed).system()));
    group.bench_function("find_by_name_miss_256", |b| {
        b.iter(|| black_box(&parsed).find_by_name(black_box("zzzz")));
    });
    group.bench_function("find_anywhere_miss_256", |b| {
        b.iter(|| black_box(&parsed).find_anywhere(black_box("zzzz")));
    });
    group.finish();
}

fn bench_live(c: &mut Criterion) {
    if get_devices().is_err() {
        eprintln!("live: no /sys here -- skipping");
        return;
    }
    let mut group = c.benchmark_group("live");
    group
        .sample_size(20)
        .measurement_time(Duration::from_secs(5));

    group.bench_function("get_devices", |b| b.iter(|| get_devices().unwrap()));

    // reference only -- what shelling out used to cost
    let lsblk_ok = Command::new("lsblk")
        .args(["--json", "--bytes"])
        .output()
        .is_ok_and(|o| o.status.success());
    if lsblk_ok {
        group.bench_function("lsblk_spawn_reference", |b| {
            b.iter(|| {
                Command::new("lsblk")
                    .args(["--json", "--bytes"])
                    .output()
                    .unwrap()
                    .stdout
            });
        });
    } else {
        eprintln!("live: lsblk not available -- skipping reference bench");
    }
    group.finish();
}

/// fixed cost yardsticks -- these must never change. cpu for the in memory
/// filters, syscall for the walk which is nothing but syscalls
fn bench_calib(c: &mut Criterion) {
    let mut group = c.benchmark_group("calib");

    // fnv over 64k, ~50us of pure alu work
    let buf: Vec<u8> = (0..65_536u32)
        .map(|i| (i.wrapping_mul(2_654_435_761) >> 24) as u8)
        .collect();
    group.bench_function("cpu", |b| {
        b.iter(|| {
            let mut h: u64 = 0xcbf2_9ce4_8422_2325;
            for &byte in black_box(&buf) {
                h ^= u64::from(byte);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
            h
        });
    });

    // 64 stats of one file -- same kind of work as the walk, none of the
    // walk's code
    let r = FakeRoot::new("calib");
    r.file("sys/block/x", "");
    let p = r.path().join("sys/block/x");
    group.bench_function("syscall", |b| {
        b.iter(|| {
            for _ in 0..64 {
                black_box(std::fs::metadata(black_box(&p)).is_ok());
            }
        });
    });

    group.finish();
}

criterion_group!(benches, bench_calib, bench_walk, bench_filters, bench_live);
criterion_main!(benches);
