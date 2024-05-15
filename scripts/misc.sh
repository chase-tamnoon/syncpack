#!/usr/bin/env bash

(RUST_BACKTRACE=1 cd fixtures/misc && cargo run -- lint --source 'package.json')
