#!/usr/bin/env python3
"""Licence gate: fail on any missing licence or any GPL-family licence
expression in the resolved dependency graph (normal, dev and build).

Runs `cargo metadata --format-version 1 --locked` and prints the offending
package plus a dependency path from the root showing how it was reached.
Stdlib only. Never weaken this gate to make a build pass; escalate instead.
"""

import json
import subprocess
import sys
from collections import deque


def main() -> int:
    proc = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--locked"],
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        print(proc.stderr, file=sys.stderr)
        return proc.returncode
    metadata = json.loads(proc.stdout)

    packages = {p["id"]: p for p in metadata["packages"]}
    resolve_nodes = {n["id"]: n for n in metadata["resolve"]["nodes"]}

    # children[pkg_id] = set of package ids it depends on (any kind).
    children: dict[str, set[str]] = {}
    for node in metadata["resolve"]["nodes"]:
        deps = children.setdefault(node["id"], set())
        for dep in node["deps"]:
            deps.add(dep["pkg"])
        deps.update(node.get("dependencies", []))

    root = metadata["resolve"]["root"]
    failures = []
    for pkg_id, pkg in sorted(packages.items(), key=lambda kv: kv[1]["name"]):
        expr = pkg.get("license")
        bad = expr is None or "GPL" in expr.upper().replace(" ", "")
        if bad:
            path = dep_path(root, pkg_id, children, resolve_nodes, packages)
            failures.append((pkg, expr, path))

    if failures:
        for pkg, expr, path in failures:
            print(
                f"FAIL: {pkg['name']} {pkg['version']} licence {expr!r}",
                file=sys.stderr,
            )
            print(f"      reached via: {path}", file=sys.stderr)
        return 1

    print(f"licence gate: {len(packages)} packages checked, all clean")
    return 0


def dep_path(root, target, children, resolve_nodes, packages) -> str:
    """BFS from the workspace root to `target`; return `a -> b -> c`."""

    def label(pkg_id):
        p = packages[pkg_id]
        return f"{p['name']} {p['version']}"

    if root == target:
        return label(root)
    queue = deque([(root, [root])])
    seen = {root}
    while queue:
        node, path = queue.popleft()
        for child in sorted(children.get(node, ())):
            if child in seen:
                continue
            if child == target:
                return " -> ".join(label(p) for p in [*path, child])
            seen.add(child)
            queue.append((child, [*path, child]))
    return f"{label(target)} (path not found in resolve graph)"


if __name__ == "__main__":
    sys.exit(main())
