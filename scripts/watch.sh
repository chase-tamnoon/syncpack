#!/usr/bin/env bash

cargo watch --clear --exec 'test -- --nocapture --color=always format'
