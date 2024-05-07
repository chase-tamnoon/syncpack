#!/usr/bin/env bash

cd fixtures/fluid-framework
cargo run -- lint --ranges --versions --source 'package.json' --source 'packages/**/package.json'
