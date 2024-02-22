#!/bin/bash -e
#
# script to run projects tests and report code coverage
#
# uses grcov (https://github.com/mozilla/grcov)

COVERAGE_DIR="target/coverage"

_tit() {
    echo
    echo "========================================"
    echo "$@"
    echo "========================================"
}

_tit "installing requirements"
rustup component add llvm-tools-preview
cargo install grcov

_tit "gathering coverage info"
# enable code coverage instrumentation and set per-test profile file name
export RUSTFLAGS="-Cinstrument-coverage"
export RUSTDOCFLAGS="-Cinstrument-coverage"
export LLVM_PROFILE_FILE="$COVERAGE_DIR/%p-%m.profraw"
# run tests
rm -rf $COVERAGE_DIR && mkdir -p $COVERAGE_DIR
cargo test --no-fail-fast || true

_tit "generating coverage report"
grcov $COVERAGE_DIR \
    -s . \
    --binary-path target/debug/ \
    --output-types html \
    --branch \
    --ignore 'target/*' \
    --ignore-not-existing \
    -o $COVERAGE_DIR/

## show html report location
echo "generated html report: $COVERAGE_DIR/html/index.html"
