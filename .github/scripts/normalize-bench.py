#!/usr/bin/env python3
"""turn criterion bencher output into a machine normalized series.

github hosted runners are ~2x apart in speed class, so raw ns across runs
mostly records which machine the job landed on. every bench gets divided by
a calibration bench from the same run that does fixed work no code change
touches: the walk and live groups by calib/syscall, the filters by calib/cpu.
output is github-action-benchmark's customSmallerIsBetter json.

usage: normalize-bench.py output.txt > normalized.json
"""
import json
import re
import sys

LINE = re.compile(r"^test (\S+) \.\.\. bench:\s+([\d,]+) ns/iter \(\+/- ([\d,]+)\)")

raw = {}
for line in open(sys.argv[1], encoding="utf-8"):
    m = LINE.match(line)
    if m:
        raw[m.group(1)] = (int(m.group(2).replace(",", "")), int(m.group(3).replace(",", "")))

cpu = raw.get("calib/cpu", (0, 0))[0]
sysc = raw.get("calib/syscall", (0, 0))[0]
if not cpu or not sysc:
    sys.exit("calibration benches missing from output -- cannot normalize")


def yardstick(name):
    group = name.split("/", 1)[0]
    return ("calib/cpu", cpu) if group == "filters" else ("calib/syscall", sysc)


out = []
for name, (ns, err) in raw.items():
    if name.startswith("calib/"):
        continue
    ref_name, ref = yardstick(name)
    out.append({
        "name": name,
        "unit": f"x {ref_name}",
        "value": round(ns / ref, 4),
        "range": f"± {round(err / ref, 4)}",
        "extra": f"raw {ns:,} ns/iter, {ref_name} {ref:,} ns/iter on this runner",
    })

json.dump(out, sys.stdout, indent=2)
print()
