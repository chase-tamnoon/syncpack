#!/usr/bin/env bash

cd fixtures/fluid-framework
RUST_BACKTRACE=1 cargo run -- lint --ranges --versions --source 'package.json' --source 'packages/**/package.json'
