#!/usr/bin/env bash
set -e
cargo build --release
install target/release/cargo-craft ~/opt/libexec/cargo-craft
