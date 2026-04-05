#!/usr/bin/env python3
"""Generate Makefile.crates from Cargo.lock for FreeBSD port."""
import re

with open("Cargo.lock") as f:
    content = f.read()

packages = re.findall(r'\[\[package\]\]\nname = "(.+?)"\nversion = "(.+?)"', content)
excluded = {"flume-core", "flume-tui"}
crates = sorted([f"{name}-{ver}" for name, ver in packages if name not in excluded])

lines = ["CARGO_CRATES=\t" + crates[0] + " \\"]
for c in crates[1:-1]:
    lines.append("\t\t" + c + " \\")
lines.append("\t\t" + crates[-1])
print("\n".join(lines))
