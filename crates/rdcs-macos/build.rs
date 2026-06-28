// Copyright 2024 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

fn main() {
    // Link ApplicationServices for AXIsProcessTrusted (accessibility permission).
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        println!("cargo:rustc-link-lib=framework=ApplicationServices");
    }
}
