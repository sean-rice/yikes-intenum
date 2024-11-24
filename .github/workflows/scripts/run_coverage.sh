#!/bin/bash
set -xeu

RUSTFLAGS="-Cinstrument-coverage" cargo test --lib

if [ "${GRCOV_LLVM_PATH:-}" != '' ]; then
    grcov . --binary-path ./target/debug/ -s . -t html --branch --ignore-not-existing -o ./coverage/ --excl-start "#\[cfg\(test\)\]" --keep-only **/src/**/* --llvm-path "$GRCOV_LLVM_PATH"
else
    grcov . --binary-path ./target/debug/ -s . -t html --branch --ignore-not-existing -o ./coverage/ --excl-start "#\[cfg\(test\)\]" --keep-only **/src/**/*
fi
