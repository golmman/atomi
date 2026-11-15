#!/bin/sh
# Plan 40 — item 48 Phase A scratch probe. One process at a time.
# Phase 0 is standalone: cd phase0 && cargo run --release.
set -eu
FEN='rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1'
CONTROL=${CONTROL:-target/release/atomi}
CANDIDATE=${CANDIDATE:-/tmp/atomi-plan40/target/release/atomi}
echo "control-1"
"$CONTROL" prove "$FEN" --guide hand --memory 192 --seed 0 --evals 5000000
echo "terms-v1"
ATOMI_GUIDE_EVAL=rich "$CANDIDATE" prove "$FEN" --guide hand --memory 192 --seed 0 --evals 5000000
echo "control-2"
"$CONTROL" prove "$FEN" --guide hand --memory 192 --seed 0 --evals 5000000
