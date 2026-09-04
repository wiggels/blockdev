//! fake sysroot builder shared by the integration tests and benches
//!
//! the walker only reads file names and contents so a fake tree is plain
//! files -- no symlinks, no device nodes. `sys/block/<name>` can be an empty
//! file, the real data lives under `sys/class/block/<name>/`

#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

pub struct FakeRoot(PathBuf);

impl FakeRoot {
    pub fn new(tag: &str) -> Self {
        let p = std::env::temp_dir().join(format!(
            "blockdev-fake-{tag}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        Self(p)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }

    pub fn file(&self, rel: &str, content: &str) -> &Self {
        let p = self.0.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, content).unwrap();
        self
    }

    pub fn dir(&self, rel: &str) -> &Self {
        fs::create_dir_all(self.0.join(rel)).unwrap();
        self
    }

    /// whole device -- sys/block entry + class/block dir w/ the basics
    pub fn device(&self, name: &str, majmin: &str, sectors: u64) -> &Self {
        self.file(&format!("sys/block/{name}"), "")
            .file(&format!("sys/class/block/{name}/dev"), majmin)
            .file(
                &format!("sys/class/block/{name}/size"),
                &sectors.to_string(),
            )
            .file(&format!("sys/class/block/{name}/ro"), "0")
            .file(&format!("sys/class/block/{name}/removable"), "0")
            .dir(&format!("sys/class/block/{name}/holders"))
            .dir(&format!("sys/class/block/{name}/slaves"))
    }

    /// partition under a whole device
    pub fn part(&self, disk: &str, name: &str, majmin: &str, sectors: u64) -> &Self {
        let base = format!("sys/class/block/{disk}/{name}");
        self.file(&format!("{base}/dev"), majmin)
            .file(&format!("{base}/size"), &sectors.to_string())
            .file(&format!("{base}/ro"), "0")
            .file(&format!("{base}/start"), "2048")
            .dir(&format!("{base}/holders"))
    }

    /// `holder` sits on top of `dev` -- dev may be a partition path like sda/sda1
    pub fn holds(&self, dev: &str, holder: &str) -> &Self {
        self.file(&format!("sys/class/block/{dev}/holders/{holder}"), "")
            .file(
                &format!(
                    "sys/class/block/{holder}/slaves/{}",
                    dev.rsplit('/').next().unwrap()
                ),
                "",
            )
    }

    pub fn dm(&self, kname: &str, majmin: &str, sectors: u64, name: &str, uuid: &str) -> &Self {
        self.device(kname, majmin, sectors)
            .file(
                &format!("sys/class/block/{kname}/dm/name"),
                &format!("{name}\n"),
            )
            .file(
                &format!("sys/class/block/{kname}/dm/uuid"),
                &format!("{uuid}\n"),
            )
    }

    pub fn md(&self, name: &str, majmin: &str, sectors: u64, level: &str) -> &Self {
        self.device(name, majmin, sectors).file(
            &format!("sys/class/block/{name}/md/level"),
            &format!("{level}\n"),
        )
    }

    /// udev db entry for a maj:min -- the `E:` lines udevd writes
    pub fn udev(&self, majmin: &str, props: &[(&str, &str)]) -> &Self {
        let body = props.iter().fold(String::new(), |mut acc, (k, v)| {
            use std::fmt::Write as _;
            let _ = writeln!(acc, "E:{k}={v}");
            acc
        });
        self.file(
            &format!("run/udev/data/b{majmin}"),
            &format!("I:123456\nS:disk/by-id/x\n{body}G:systemd\n"),
        )
    }

    pub fn mounts(&self, mountinfo: &str, swaps: &str) -> &Self {
        self.file("proc/self/mountinfo", mountinfo).file(
            "proc/swaps",
            &format!("Filename\t\tType\t\tSize\tUsed\tPriority\n{swaps}"),
        )
    }

    /// n disks each w/ one partition, human sized like the old parse bench --
    /// the deterministic workload for perf budgets and criterion
    pub fn many_disks(&self, n: u32) -> &Self {
        for i in 0..n {
            let disk = format!("nvme{i}n1");
            self.device(&disk, &format!("259:{}", i * 2), 7_516_192_768)
                .part(
                    &disk,
                    &format!("{disk}p1"),
                    &format!("259:{}", i * 2 + 1),
                    937_703_088,
                );
        }
        self.mounts("20 1 259:1 / / rw - ext4 /dev/nvme0n1p1 rw\n", "")
    }
}

impl Drop for FakeRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
