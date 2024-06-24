#!/usr/bin/env bash

(RUST_BACKTRACE=1 cd fixtures/fluid-framework && cargo run -- lint --versions)
