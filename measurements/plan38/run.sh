#!/bin/sh
# Plan 38 (item 52 Phase A) — reproduce the bounded-length measurements.
# One process at a time; the control is the same binary with no --max-length.
set -eu
FEN='rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1'
BIN=${BIN:-target/release/atomi}
cargo build --release
for L in '' 20 22 24; do
  if [ -z "$L" ]; then
    echo "control 5M"
    "$BIN" prove "$FEN" --guide hand --memory 192 --seed 0 --evals 5000000
  else
    echo "L=$L 5M"
    "$BIN" prove "$FEN" --guide hand --memory 192 --seed 0 --evals 5000000 --max-length "$L"
  fi
done
for L in '' 20 22 24; do
  if [ -z "$L" ]; then
    echo "control 120s"
    "$BIN" prove "$FEN" --guide hand --memory 192 --seed 0 --seconds 120
  else
    echo "L=$L 120s"
    "$BIN" prove "$FEN" --guide hand --memory 192 --seed 0 --seconds 120 --max-length "$L"
  fi
done
