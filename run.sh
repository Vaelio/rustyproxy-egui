#!/bin/sh

RUSTFLAGS="-C target_cpu=native -C opt-level=3 -C debuginfo=0" cargo run -r -- $@
