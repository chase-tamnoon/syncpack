#!/usr/bin/env bash

# https://lib.rs/crates/cargo-llvm-cov
brew install cargo-llvm-cov

cargo llvm-cov --html

echo "open target/llvm-cov/html/index.html"
