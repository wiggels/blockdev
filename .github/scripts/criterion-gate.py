#!/usr/bin/env python3
"""fail if any criterion bench regressed past a threshold vs its baseline.

reads criterion's human output on stdin -- the block per bench looks like

    walk/disks/16           time:   [454.89 us 459.47 us 463.83 us]
                            change: [-22.078% -21.020% -20.201%] (p = 0.00 < 0.05)
                            Performance has improved.

takes the middle estimate of the change line. criterion prints a unicode
minus so normalize that. threshold is a percent, default 25.
"""
import re
import sys

threshold = float(sys.argv[1]) if len(sys.argv) > 1 else 25.0
bench = None
rows = []
worst = []
for raw in sys.stdin:
    line = raw.replace("−", "-").rstrip()
    # short ids share a line with time:, long ones get their own line first
    m = re.match(r"^(\S+)\s+time:", line)
    if m:
        bench = m.group(1)
        continue
    if re.match(r"^[\w./-]+/[\w./-]+$", line):
        bench = line
        continue
    m = re.search(r"change:\s*\[\s*([-+]?[\d.]+)%\s+([-+]?[\d.]+)%\s+([-+]?[\d.]+)%", line)
    if m and bench:
        mid = float(m.group(2))
        rows.append((bench, mid))
        if mid > threshold:
            worst.append((bench, mid))
        bench = None

if not rows:
    print("no change lines found -- was --baseline given and does it exist?")
    sys.exit(2)

print(f"{'bench':<40} {'vs base':>10}")
for name, mid in rows:
    print(f"{name:<40} {mid:>+9.1f}%")

if worst:
    print(f"\nregressed past +{threshold:.0f}% on the same runner:")
    for name, mid in worst:
        print(f"  {name}: {mid:+.1f}%")
    sys.exit(1)
print(f"\nnothing past +{threshold:.0f}%")
