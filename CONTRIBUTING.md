# Contributing

Thanks for considering a contribution to `blockdev`. This covers the project
conventions; if anything is unclear, open an issue and ask.

## Development setup

```sh
git clone https://github.com/wiggels/blockdev
cd blockdev
cargo test
```

The crate targets the **2024 edition** with an MSRV of **1.85** (the floor
edition 2024 itself needs). CI verifies the crate still builds on 1.85, so
don't reach for newer features without bumping `rust-version` in
`Cargo.toml` -- that is a breaking change for downstream consumers.

Everything is Linux sysfs, so `get_devices()` only works on Linux. The
walker itself is plain file reads, which is why the test suite builds fake
sysroots in a temp dir and runs anywhere, including macOS CI. The two tests
that need a real `/sys` (and one of them `lsblk`) are `#[ignore]`d and CI
runs them explicitly on the Linux job.

## Project layout

```txt
src/lib.rs        Types, filters, get_devices / get_devices_at.
src/sysfs.rs      The walk -- sysfs and mountinfo reading, mirrors lsblk's rules.
tests/common/     FakeRoot -- builds fake sysroots for tests and benches.
tests/layouts.rs  Snapshot tests over real-shaped machine layouts (insta).
tests/lsblk_equivalence.rs  Ignored live diff against lsblk --json --bytes.
tests/perf_budgets.rs       Wall-clock budgets, run under plain cargo test.
benches/devices.rs          Criterion regression benchmarks.
docs/             Design notes and investigations.
.github/          CI, bench, release, audit, semver, coverage, dependabot configs.
```

## Pull request checklist

Before opening a PR, run locally:

```sh
cargo fmt --all
cargo clippy --all-targets -- -D warnings -W clippy::pedantic
cargo test --all-targets --locked
cargo test --all-targets --locked --features serde
cargo test --release --test perf_budgets -- --nocapture
cargo doc --no-deps --all-features
```

The same checks run in CI. PRs need every one of them green to merge.

### Commit messages

Use [Conventional Commits](https://www.conventionalcommits.org/). `release-plz`
parses commit messages to generate changelog entries and version bumps, so:

* `feat: ...` -> minor bump, listed under "Added"
* `fix: ...` -> patch bump, listed under "Fixed"
* `perf: ...` -> patch bump, listed under "Performance"
* `docs: ...` -> no version change, listed under "Documentation"
* `chore: ...` / `ci: ...` -> no version change, skipped from changelog
* Anything with `BREAKING CHANGE:` in the body -> major bump

### Snapshot tests

We use [`insta`](https://insta.rs/) for snapshot tests over a handful of
real-shaped machine layouts (RAID root, LUKS+LVM, EC2, workstation) built
as fake sysroots in `tests/layouts.rs`. When you change something that
affects the walk's output, expect a snapshot to fail. Review the diff,
decide whether the change is intentional, and accept it:

```sh
cargo install cargo-insta   # one time
cargo insta review
```

The accepted `.snap` files get committed alongside the code change.

### Matching lsblk

The walker is meant to give the same answer as `lsblk --json --bytes`. If
you touch `src/sysfs.rs`, read the matching code in util-linux
(`lsblk-cmd/lsblk.c`, `lsblk-cmd/mnt.c`, `lib/sysfs.c`) and run the live
diff on a Linux box:

```sh
cargo test --test lsblk_equivalence -- --ignored
```

A layout that lsblk handles and we do not is a bug; add it as a fake
sysroot in `tests/layouts.rs` along with the fix.

### Benchmarks

The repo has three layers of perf regression defense:

1. **`tests/perf_budgets.rs`** -- wall-clock budgets enforced by `cargo test`.
   Catastrophic regressions fail there.
2. **`benches/devices.rs`** -- criterion benchmarks over fake sysroots plus
   a live walk. Run locally with
   `cargo bench --bench devices -- --save-baseline main` before changes,
   then `cargo bench --bench devices -- --baseline main` after to compare.
3. **CI bench workflow** -- runs against `main` on every PR and fails the
   build if anything regresses by more than 25%. History is published to
   GitHub Pages.

If you're optimising, please include before/after criterion output in the
PR description.

## Release process

Releases are automated via [`release-plz`](https://release-plz.dev/):

1. Commits land on `main` using Conventional Commits.
2. `release-plz` opens a "Release v0.x.y" PR with a version bump and a
   draft changelog entry. The maintainer reviews and merges it.
3. The merge triggers `release-plz release`, which tags the commit,
   publishes to crates.io, and creates a GitHub Release.

Don't bump versions or write `CHANGELOG.md` entries by hand -- let
release-plz manage them. If a bump needs manual adjustment, do it on the
release PR, not in a separate commit.

## License

By contributing, you agree that your contributions will be licensed under
the same MIT licence that covers the rest of the project.
