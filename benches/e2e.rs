//! end to end benches -- the whole `get_devices()` request, not just the parse
//!
//! parse.rs measures serde over fixtures. that is the part we control but it
//! is not what a caller pays: `get_devices` forks lsblk, waits for it to walk
//! sysfs and print json, then parses. these benches split that cost so the
//! numbers say where time actually goes:
//!
//! * `spawn_only` -- fork+exec lsblk and read stdout, no parse. the floor of
//!   any approach that shells out
//! * `parse_live_output` -- parse exactly what this machine's lsblk emitted
//! * `full_request` -- `get_devices()` as shipped
//!
//! needs a real lsblk. if it is missing -- macos, minimal containers -- the
//! group registers nothing and says so, rather than failing cargo bench
//!
//! ms scale per iter so sample size is small to keep ci runtime sane

use std::hint::black_box;
use std::process::Command;
use std::time::Duration;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};

/// one probe run -- Some(stdout) if lsblk works here
fn probe_lsblk() -> Option<Vec<u8>> {
    let out = Command::new("lsblk")
        .arg("--json")
        .arg("--bytes")
        .output()
        .ok()?;
    out.status.success().then_some(out.stdout)
}

fn bench_e2e(c: &mut Criterion) {
    let Some(live) = probe_lsblk() else {
        eprintln!("e2e: lsblk not available on this machine -- skipping get_devices benches");
        return;
    };
    let live_str = String::from_utf8(live).expect("lsblk output is utf8");

    let mut group = c.benchmark_group("get_devices");
    group.throughput(Throughput::Bytes(live_str.len() as u64));

    group.bench_function("spawn_only", |b| {
        b.iter(|| {
            let out = Command::new("lsblk")
                .arg("--json")
                .arg("--bytes")
                .output()
                .unwrap();
            black_box(out.stdout)
        });
    });

    group.bench_function("parse_live_output", |b| {
        b.iter(|| blockdev::parse_lsblk(black_box(&live_str)).unwrap());
    });

    group.bench_function("full_request", |b| {
        b.iter(|| blockdev::get_devices().unwrap());
    });

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(20)
        .measurement_time(Duration::from_secs(5))
        .warm_up_time(Duration::from_secs(1));
    targets = bench_e2e
}
criterion_main!(benches);
