# Blockdev Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [0.6.0](https://github.com/wiggels/blockdev/compare/v0.5.0...v0.6.0) - 2026-09-04

### Added

- Add uuid, partuuid, fstype, label, partlabel, wwn, serial, model

### Fixed

- Clippy assigning_clones in the equivalence test

### Performance

- Gate identifier lookups on the dir listing, prefer unmangled model

## [0.5.0](https://github.com/wiggels/blockdev/compare/v0.4.2...v0.5.0) - 2026-09-04

### Added

- [**breaking**] Walk `/sys` and `/proc` directly. `get_devices()` no longer spawns `lsblk` or parses JSON; it reads the same sysfs files `lsblk` does, matches its output, and is several times faster. Zero runtime dependencies.
- `get_devices_at(sysroot)`, the equivalent of `lsblk --sysroot`, for bind-mounted host trees and tests.
- Optional `serde` feature providing `Serialize`/`Deserialize` on every type; `MajMin` as the `major:minor` string.
- `DeviceType` implements an infallible `FromStr` and is `Copy`.

### Changed

- [**breaking**] `BlockDevices.blockdevices` renamed to `devices`.
- [**breaking**] `BlockDevice.children` is `Vec<BlockDevice>` instead of `Option<Vec<BlockDevice>>`.
- [**breaking**] `BlockDevice.mountpoints` is `Vec<String>` instead of `Vec<Option<String>>`; empty when unmounted.
- [**breaking**] `BlockDevError` is `#[non_exhaustive]` with a single `Io { path, source }` variant.

### Removed

- [**breaking**] `parse_lsblk`, `BlockDevice::children_iter`, `BlockDevice::active_mountpoints`, `BlockDevice::active_mountpoints_iter`, and the `serde_json` and `thiserror` dependencies.

### Documentation

- `docs/sysfs-backend.md`: why there is no lsblk C API to call, what the walk reads, and what it does not cover.

### Fixed

- Satisfy clippy 1.98 map_unwrap_or on the slaves check

## [0.4.2](https://github.com/wiggels/blockdev/compare/v0.4.1...v0.4.2) - 2026-09-04

### Performance

- Drop `serde_json::Value` round trips in the `size` and `mountpoints` deserializers in favor of direct visitors, and parse `maj:min` and human-readable sizes with exact integer math.

## [0.4.1](https://github.com/wiggels/blockdev/compare/v0.4.0...v0.4.1) - 2026-09-04

### Documentation

- Keep perf section neutral until the parser changes land
- Add CONTRIBUTING, ci badges, perf and regression sections

### Tests

- Add perf budgets, criterion 0.8 with throughput, msrv 1.85

### Bench

- Add end to end get_devices benchmark

### Build

- Switch release flow to release-plz

## [0.4.0](https://github.com/wiggels/blockdev/compare/v0.3.1...0.4.0) - 2026-05-14

- ⛰️ Add more tests/benches and fix perf ([653e4f5](https://github.com/wiggels/blockdev/commit/653e4f5c079a00b992514fa5e33b940a3cf365fa))

## [0.3.1](https://github.com/wiggels/blockdev/compare/v0.3.0...v0.3.1) - 2026-01-08

- 🏗️ Release blockdev version 0.3.1 ([8cd65c0](https://github.com/wiggels/blockdev/commit/8cd65c0fefd5780085dbee33cbb770520eee8175))
- 🐛 Update readme and fix cliff links ([015b24f](https://github.com/wiggels/blockdev/commit/015b24f15d0abe87e50af8778ee97c2091363d57))

## [0.3.0](https://github.com/wiggels/blockdev/compare/v0.2.1...v0.3.0) - 2026-01-08

- ⛰️ Parse maj_min into MajMin struct with major/minor fields ([f611f02](https://github.com/wiggels/blockdev/commit/f611f02207736f9fbe0e140b48c43d7da2a11b96))
- ⛰️ Replace device_type String with DeviceType enum ([8e89660](https://github.com/wiggels/blockdev/commit/8e89660af78f935be4f2eb2c8c0e09536c432302))
- ⛰️ Parse size as bytes (u64) instead of string ([477af6e](https://github.com/wiggels/blockdev/commit/477af6e36f7400eee6e0431d28f2a9728dc817c4))
- ⛰️ Replace Box<dyn Error> with custom BlockDevError type ([bdbd46a](https://github.com/wiggels/blockdev/commit/bdbd46a6861141a4159e0b86aa2fc917887f5045))
- 🏗️ Release blockdev version 0.3.0 ([1035f74](https://github.com/wiggels/blockdev/commit/1035f746c9f550c3284757a0817e6ccee9933cf5))

## [0.2.1](https://github.com/wiggels/blockdev/compare/v0.2.0...v0.2.1) - 2026-01-08

- 🏗️ Release blockdev version 0.2.1 ([761c6dd](https://github.com/wiggels/blockdev/commit/761c6dd8896c4ca1b9bc198d815c684c96263564))
- 🐛 Fix changelog generation ([fc7d1aa](https://github.com/wiggels/blockdev/commit/fc7d1aa411804fa09cfa87b173cee5bfc5bbb734))

## [0.2.0](https://github.com/wiggels/blockdev/compare/v0.1.2...v0.2.0) - 2026-01-08

- ⛰️ *(api)* Add utility methods and improve idiomatic Rust patterns ([dbbae7d](https://github.com/wiggels/blockdev/commit/dbbae7da056060adcdec4e0b74cb429db30c921f))
- 🏗️ Release blockdev version 0.2.0 ([795c517](https://github.com/wiggels/blockdev/commit/795c517154ef85f5a9dc9104becc938c6d1638da))
- 🏗️ Add placeholder changelog ([fe3e458](https://github.com/wiggels/blockdev/commit/fe3e458d2eadb76ab2a7f8d83f53896f17652dd3))
- 🏗️ Update deps and add release helpers ([0291090](https://github.com/wiggels/blockdev/commit/0291090c38741e3ecc5704213ed99c02fa0b99a2))

## [0.1.2](https://github.com/wiggels/blockdev/releases/tag/v0.1.2) - 2025-02-24

- 🏗️ Release blockdev version 0.1.2 ([819e998](https://github.com/wiggels/blockdev/commit/819e998b5cf20436b9ce4493f1a3ca3c54328ba4))
- 🏗️ Impl IntoIterator for BlockDevices ([31af4a5](https://github.com/wiggels/blockdev/commit/31af4a559079b69ccebfb8d69adf0ebcc009a5e3))
- 🏗️ Add LICENSE ([600f5d0](https://github.com/wiggels/blockdev/commit/600f5d0750a0376798b9b9a8aec0ab1933e535b3))
- 🏗️ Release blockdev version 0.1.1 ([f3e51d6](https://github.com/wiggels/blockdev/commit/f3e51d681893f2213926d88262e2bba271047584))
- 🏗️ Update category ([8f78cf5](https://github.com/wiggels/blockdev/commit/8f78cf5675c45ff7df5e2a6f39c3b68785b0ffbd))
- 🏗️ Updated keywords ([90040b4](https://github.com/wiggels/blockdev/commit/90040b43b5e70df918a85ee0174193b34059c2f4))
- 🏗️ Init commit ([cd6550c](https://github.com/wiggels/blockdev/commit/cd6550c8cc3a9f22d554d97f68607506f52a0f9f))
- 🐛 Fix branch ref ([a372a8c](https://github.com/wiggels/blockdev/commit/a372a8cd8f7ea4bb09687418372a4fc2bb500916))
- 📦 Update rust for 2024 ([e4f27f3](https://github.com/wiggels/blockdev/commit/e4f27f318e8c861f4132f3f3b91b07ccc7bc084d))
<!-- generated by git-cliff -->
