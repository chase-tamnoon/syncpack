#!/usr/bin/env bash

RUST_BACKTRACE=1 cargo watch --clear --exec 'test -- --nocapture --color=always'
