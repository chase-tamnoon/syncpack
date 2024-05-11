#!/usr/bin/env bash

cd fixtures/misc
RUST_BACKTRACE=1 cargo run -- lint --source 'package.json'
