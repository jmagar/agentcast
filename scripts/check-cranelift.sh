#!/usr/bin/env bash
set -euo pipefail

rustup toolchain install nightly >/dev/null
rustup component add rustc-codegen-cranelift --toolchain nightly >/dev/null

nightly_rustc="$(rustup which --toolchain nightly rustc)"

RUSTC="$nightly_rustc" \
RUSTFLAGS="-Zcodegen-backend=cranelift" \
rustup run nightly cargo check --workspace
