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

`get_devices()` shells out to `lsblk`, which only exists on Linux. Everything
else (`parse_lsblk`, the types, the filters) is pure and runs anywhere. The
one test that actually invokes `lsblk` is `#[ignore]`d and CI runs it
explicitly on the Linux job.

## Project layout

```txt
src/lib.rs        The whole library -- types, deserializers, filters, get_devices.
tests/            Snapshot tests (insta) over real-shaped lsblk fixtures, perf budgets.
benches/          Criterion regression benchmarks + the small.json fixture.
.github/          CI, bench, release, audit, semver, coverage, dependabot configs.
```

## Pull request checklist

Before opening a PR, run locally:

```sh
cargo fmt --all
cargo clippy --all-targets -- -D warnings -W clippy::pedantic
cargo test --all-targets --locked
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
real-shaped `lsblk` fixtures (RAID root, legacy single `mountpoint`,
LUKS+LVM, EC2). When you change something that affects parsed output, expect
a snapshot to fail. Review the diff, decide whether the change is
intentional, and accept it:

```sh
cargo install cargo-insta   # one time
cargo insta review
```

The accepted `.snap` files get committed alongside the code change.

### Benchmarks

The repo has three layers of perf regression defense:

1. **`tests/perf_budgets.rs`** -- wall-clock budgets enforced by `cargo test`.
   Catastrophic regressions fail there.
2. **`benches/parse.rs`** -- criterion micro-benchmarks. Run locally with
   `cargo bench --bench parse -- --save-baseline main` before changes,
   then `cargo bench --bench parse -- --baseline main` after to compare.
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
