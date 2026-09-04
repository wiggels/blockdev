# sysfs backend investigation

Question: util-linux moved lsblk into its own `lsblk-cmd/` directory. Can
`blockdev` call that C code directly and skip JSON deserialization?

Short answer: there is no C API to call, and deserialization was never the
cost. The process spawn is. A pure Rust walk of `/sys` and `/proc`, which is
exactly what lsblk does internally, removes both the spawn and the JSON step
and is about 5x faster end to end.

## What lsblk-cmd actually is

`lsblk-cmd/` in util-linux master is the lsblk *command*, relocated from
`misc-utils/`. Its build files define `bin_PROGRAMS += lsblk` and nothing
else. There is no `liblsblk`, no installed header, no stable symbol table.
`lsblk.h` is an internal header that pulls in `c.h`, `list.h`, `debug.h`,
and a global `struct lsblk *lsblk` handler that `main()` owns.

Two further problems even if we vendored the sources:

- **License.** lsblk is GPL-2.0-or-later. Linking it into an MIT crate would
  relicense every downstream binary. libmount and libsmartcols, which it
  depends on, are LGPL, which is workable, but lsblk.c itself is not.
- **Dependency weight.** lsblk links libblkid, libmount, libsmartcols,
  libtcolors, and optionally libudev. That is the whole util-linux build
  system for a crate that currently has three pure Rust deps.

So "direct C API" is off the table on its own terms.

## Where the time really goes

The end-to-end bench in `benches/e2e.rs` splits a `get_devices()` call:

| step | time |
|---|---|
| fork+exec lsblk, wait, read stdout | ~3.8 ms |
| parse that output with `parse_lsblk` | ~4 µs |
| `get_devices()` total | ~3.5 ms |

The JSON parse is roughly 0.1% of the request. Even a zero-cost
deserializer would be invisible. The only lever that matters is not
starting a process.

## What lsblk does under the hood

Reading `lsblk-cmd/lsblk.c`, `devtree.c`, `mnt.c`, and `lib/sysfs.c`:

- Roots come from `readdir(/sys/block)`. Major 1 (ram disks) is excluded by
  default. Anything with a non-empty `slaves/` directory is "in the middle"
  of a stack and is shown under its parent instead of as a root. Zero-size
  loop devices with no `loop/backing_file` are hidden.
- Partitions are subdirectories of the whole-disk directory named
  `<disk>N` or `<disk>pN`. Anything stacked on a device is listed by name in
  its `holders/` directory and is resolved through `/sys/class/block/<name>`.
- `size` is 512-byte sectors, shifted left by 9. `ro` and `removable` are
  the sysfs attributes; a partition inherits `removable` from its disk.
- `type` is: `part` for partitions; the `dm/uuid` prefix lowercased for
  `dm-*` (LVM, CRYPT, mpath, with the kpartx `partN-` hack folding to
  `part`); `loop`; `md/level` for `md*`; the SCSI `device/type` mapping
  otherwise, defaulting to `disk`.
- Mountpoints are every `/proc/self/mountinfo` line matching by `maj:min`
  or by source path, newest first, then `/proc/swaps` for `[SWAP]`.
- The whole tree is sorted by `maj:min` because `/sys` stopped being sorted
  in Linux 4.8.
- Names swap `!` for `/` (`cciss!c0d0` is `/dev/cciss/c0d0`), and `dm-N`
  devices are shown by their `dm/name`.

`src/sysfs.rs` reproduces all of that. `sysfs_backend_matches_lsblk` in
`lib.rs` asserts the two backends produce an identical `BlockDevices` on
the running machine, and CI runs it on the Linux job. Fixture tests in
`sysfs.rs` cover the stacked layouts a CI box does not have: RAID1 over
partitions with swap, LUKS then LVM, hidden ram and loop devices, removable
inheritance, SCSI rom, and escaped mountpoints.

## Result

Same bench, same machine, with the sysfs backend added:

| path | time | vs lsblk |
|---|---|---|
| `get_devices()` (spawn lsblk + parse) | ~3.45 ms | 1x |
| `get_devices_sysfs()` | ~0.61 ms | 5.6x faster |

The remaining 600 µs is roughly 300 syscalls. This box is a Firecracker VM
where each syscall costs 3 to 4 µs; on bare metal or a normal KVM guest the
same walk should land near 100 to 150 µs, so the gap to lsblk will be wider
there, not narrower. The walk already uses one open/read/close per
attribute instead of the size-probing `read_to_string`, and skips the
partition check on roots. Cutting further means `openat` relative to a
directory fd, which needs `libc` or `rustix`; not worth a dependency for
the current numbers.

## What is not covered

- Multi-device filesystems where only one member appears in mountinfo
  (btrfs RAID, ZFS pools). lsblk has special-case grouping for these.
- The `BLKROGET` ioctl fallback when the `ro` attribute is missing. Every
  kernel this crate can realistically run on has the attribute.
- lsblk's udev/blkid property probing. None of those columns are in
  `BlockDevice`.

## Recommendation

Keep `get_devices()` on lsblk as the default for now. It is the reference
behaviour and works on any distro's util-linux version. Ship
`get_devices_sysfs()` alongside it. If the equivalence test stays green
across a few releases and real-world layouts, flip the default in a minor
bump and keep the lsblk path as the fallback when `/sys/block` is
unreadable.

Adding a `Sysfs` variant to `BlockDevError` would be a breaking change
because the enum is not `#[non_exhaustive]`, which is why
`get_devices_sysfs()` returns `std::io::Error` directly. Marking the enum
non-exhaustive is worth doing in the next minor so the default can switch
cleanly later.
