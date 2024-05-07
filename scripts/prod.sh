#!/usr/bin/env bash

cd fixtures/fluid-framework
../../target/release/syncpack lint --ranges --versions --source 'package.json' --source 'packages/**/package.json'
