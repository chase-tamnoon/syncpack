#!/usr/bin/env bash

(cd fixtures/fluid-framework && ../../target/release/syncpack lint --versions --source 'package.json' --source 'packages/**/package.json')
