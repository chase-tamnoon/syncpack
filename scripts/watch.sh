#!/usr/bin/env bash

RUST_BACKTRACE=1 cargo watch -x 'test -- --nocapture --color=always'
