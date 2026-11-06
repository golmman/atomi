#!/usr/bin/env bash
# Builds the pinned provari oracle (GPL-3.0) in a scratch directory OUTSIDE
# this repository. Nothing is vendored or committed; the binary is used only
# as an external process (`provari check`, bench reference).
#
# Usage: scripts/build_oracle.sh [--ref v0.2.0] [DIR]
#   default ref: 48686bc6c786d6ea70568f0b2ad4002cf0520206 (v0.1.0, the veto)
#   --ref v0.2.0:  224ce85dad083b1bc8fe429a9e4349de956f5305 (stretch reference)
#   DIR: scratch root, default /tmp/atomi-oracle; the clone goes to
#        DIR/provari (default ref) or DIR/provari-v0.2.0 (--ref v0.2.0).

set -euo pipefail

REPO_URL="https://github.com/aiorla/provari"
PIN_V010="48686bc6c786d6ea70568f0b2ad4002cf0520206"
PIN_V020="224ce85dad083b1bc8fe429a9e4349de956f5305"

REF="v0.1.0"
SHA="$PIN_V010"
DIR="/tmp/atomi-oracle"

while [ $# -gt 0 ]; do
    case "$1" in
        --ref)
            REF="$2"
            shift 2
            ;;
        *)
            DIR="$1"
            shift
            ;;
    esac
done

case "$REF" in
    v0.1.0) SHA="$PIN_V010"; NAME="provari" ;;
    v0.2.0) SHA="$PIN_V020"; NAME="provari-v0.2.0" ;;
    *) echo "unknown --ref $REF (expected v0.1.0 or v0.2.0)" >&2; exit 2 ;;
esac

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd -P)"
TARGET_DIR="$DIR/$NAME"
RESOLVED_TARGET="$(realpath -m "$TARGET_DIR")"
case "$RESOLVED_TARGET" in
    "$REPO_ROOT"|"$REPO_ROOT"/*)
        echo "refusing: $RESOLVED_TARGET is inside the repository" >&2
        exit 2
        ;;
esac
mkdir -p "$DIR"

if [ ! -d "$TARGET_DIR/.git" ]; then
    if [ -n "$(ls -A "$TARGET_DIR")" ]; then
        echo "refusing: $TARGET_DIR exists and is not a git clone" >&2
        exit 2
    fi
    git clone "$REPO_URL" "$TARGET_DIR"
fi

git -C "$TARGET_DIR" fetch --tags origin
git -C "$TARGET_DIR" checkout "$SHA"
HEAD="$(git -C "$TARGET_DIR" rev-parse HEAD)"
if [ "$HEAD" != "$SHA" ]; then
    echo "HEAD verification failed: $HEAD != $SHA" >&2
    exit 1
fi

cargo build --release --manifest-path "$TARGET_DIR/Cargo.toml"

BIN="$TARGET_DIR/target/release/provari"
echo "oracle built: $BIN"
"$BIN" --version
