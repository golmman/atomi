#!/bin/sh
# Plan 39 (item 52 Phase B) — Phase 0 L-curve probe on deep-win.
# One process at a time; the control is the same binary with no --max-length.
set -eu
FEN='rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1'
BIN=${BIN:-target/release/atomi}
cargo build --release
# The same-session reference: unbounded control and the single L = 24 pass.
echo "control 120s"
"$BIN" prove "$FEN" --guide hand --memory 192 --seed 0 --seconds 120
echo "L=24 120s"
"$BIN" prove "$FEN" --guide hand --memory 192 --seed 0 --seconds 120 --max-length 24
# Phase 0: small-L probes at 5 M and 120 s.
for L in 12 14 16 18; do
  echo "L=$L 5M"
  "$BIN" prove "$FEN" --guide hand --memory 192 --seed 0 --evals 5000000 --max-length "$L"
  echo "L=$L 120s"
  "$BIN" prove "$FEN" --guide hand --memory 192 --seed 0 --seconds 120 --max-length "$L"
done
