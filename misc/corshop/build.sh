#! /bin/bash
cargo build --release --target x86_64-unknown-linux-musl
mv target/x86_64-unknown-linux-musl/release/cor-shop bin/
