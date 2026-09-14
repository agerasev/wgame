#!/usr/bin/env bash
set -euo pipefail
optional=(shapes fs image typography utils)
for ((mask=0; mask<32; mask++)); do
  features=desktop
  for ((bit=0; bit<5; bit++)); do
    if ((mask & (1 << bit))); then features+=,${optional[bit]}; fi
  done
  cargo check --locked -p wgame --no-default-features --features "$features"
done
cargo check --locked -p wgame --target wasm32-unknown-unknown --no-default-features --features web
cargo check --locked -p wgame-examples --target wasm32-unknown-unknown --no-default-features --features web

cargo check --locked -p wgame-egui --no-default-features --features desktop
cargo check --locked -p wgame-egui --examples --target wasm32-unknown-unknown --no-default-features --features web
