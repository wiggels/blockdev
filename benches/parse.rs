//! criterion benches for `blockdev` -- regression detection, not micro-opt
//!
//! each bench is one user visible op with a stable name so ci can diff it
//! against the baseline stored on gh-pages. parse benches report throughput
//! in bytes so a regression reads in absolute terms
//!
//! ```sh
//! cargo bench --bench parse -- --save-baseline main
//! # ... make changes ...
//! cargo bench --bench parse -- --baseline main
//! ```
//!
//! criterion writes reports to target/criterion -- gitignored so they stay
//! local. anything over 5% slower gets flagged in the report

use std::fmt::Write as _;

use blockdev::{BlockDevices, parse_lsblk};
use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};

const SMALL_FIXTURE: &str = include_str!("fixtures/small.json");

fn make_large_fixture(n_disks: usize) -> String {
    let mut s = String::from("{\"blockdevices\":[");
    for i in 0..n_disks {
        if i > 0 {
            s.push(',');
        }
        let major = 259;
        let minor = i * 2;
        let child_minor = minor + 1;
        write!(
            s,
            "{{\"name\":\"nvme{i}n1\",\"maj:min\":\"{major}:{minor}\",\"rm\":false,\
             \"size\":\"3.5T\",\"ro\":false,\"type\":\"disk\",\"mountpoints\":[null],\
             \"children\":[\
               {{\"name\":\"nvme{i}n1p1\",\"maj:min\":\"{major}:{child_minor}\",\
                 \"rm\":false,\"size\":\"3.5T\",\"ro\":false,\"type\":\"part\",\
                 \"mountpoints\":[null]}}\
             ]}}",
        )
        .unwrap();
    }
    s.push_str("]}");
    s
}

fn make_size_string_fixture(n: usize) -> String {
    let suffixes = ["500G", "3.5T", "8M", "1.7T", "894.3G", "447.1G", "19.1G", "488M"];
    let mut s = String::from("{\"blockdevices\":[");
    for i in 0..n {
        if i > 0 {
            s.push(',');
        }
        let size = suffixes[i % suffixes.len()];
        write!(
            s,
            "{{\"name\":\"d{i}\",\"maj:min\":\"8:{i}\",\"rm\":false,\
             \"size\":\"{size}\",\"ro\":false,\"type\":\"disk\",\"mountpoints\":[null]}}"
        )
        .unwrap();
    }
    s.push_str("]}");
    s
}

fn make_byte_size_fixture(n: usize) -> String {
    let mut s = String::from("{\"blockdevices\":[");
    for i in 0..n {
        if i > 0 {
            s.push(',');
        }
        write!(
            s,
            "{{\"name\":\"d{i}\",\"maj:min\":\"8:{i}\",\"rm\":false,\
             \"size\":536870912000,\"ro\":false,\"type\":\"disk\",\"mountpoints\":[null]}}"
        )
        .unwrap();
    }
    s.push_str("]}");
    s
}

fn bench_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_lsblk");

    group.throughput(Throughput::Bytes(SMALL_FIXTURE.len() as u64));
    group.bench_function("small_realistic", |b| {
        b.iter(|| parse_lsblk(black_box(SMALL_FIXTURE)).unwrap());
    });

    let large = make_large_fixture(256);
    group.throughput(Throughput::Bytes(large.len() as u64));
    group.bench_function("large_256_disks_human_size", |b| {
        b.iter(|| parse_lsblk(black_box(&large)).unwrap());
    });

    let bytes = make_byte_size_fixture(256);
    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("large_256_disks_byte_size", |b| {
        b.iter(|| parse_lsblk(black_box(&bytes)).unwrap());
    });

    let sizes = make_size_string_fixture(1024);
    group.throughput(Throughput::Bytes(sizes.len() as u64));
    group.bench_function("size_string_heavy_1024", |b| {
        b.iter(|| parse_lsblk(black_box(&sizes)).unwrap());
    });

    group.finish();
}

fn bench_filters(c: &mut Criterion) {
    let mut group = c.benchmark_group("filters");

    let parsed: BlockDevices = parse_lsblk(SMALL_FIXTURE).unwrap();

    group.bench_function("system_filter_small", |b| {
        b.iter(|| black_box(&parsed).system());
    });

    group.bench_function("non_system_filter_small", |b| {
        b.iter(|| black_box(&parsed).non_system());
    });

    group.bench_function("find_by_name_hit", |b| {
        b.iter(|| black_box(&parsed).find_by_name(black_box("nvme3n1")));
    });

    group.bench_function("find_by_name_miss", |b| {
        b.iter(|| black_box(&parsed).find_by_name(black_box("zzzz")));
    });

    let large = make_large_fixture(256);
    let large_parsed: BlockDevices = parse_lsblk(&large).unwrap();
    group.bench_function("system_filter_256", |b| {
        b.iter(|| black_box(&large_parsed).system());
    });

    group.finish();
}

criterion_group!(benches, bench_parse, bench_filters);
criterion_main!(benches);
