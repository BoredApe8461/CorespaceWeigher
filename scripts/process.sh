#!/bin/bash

sh -c 'RUST_LOG=INFO ./target/release/processor' >> logs/processor.out 2>&1
