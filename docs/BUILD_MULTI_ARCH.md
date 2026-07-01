# Multi-Architecture Build Guide

This document describes how to build the RDCS client for both Intel Mac and Apple Silicon, with instructions for development, release, and CI/CD workflows.

## Supported Architectures

- **Apple Silicon** (arm64): M1/M2/M3/M4 and newer chips
- **Intel Mac** (x86_64): Intel Core series processors

Each architecture produces a separate, optimized binary. The CI/CD pipeline automatically builds both.

## Development Build

For local development, Flutter automatically detects and builds for the current architecture:

```bash
cd client/flutter
flutter run -d macos
```

Or build without running:

```bash
flutter build macos --debug
```

This creates an unoptimized, debug-enabled build in `build/macos/Build/Products/Debug/`.

## Release Build

Release builds are optimized and require an explicit target architecture.

### Apple Silicon (arm64)

Build on an Apple Silicon Mac:

```bash
cd client/flutter
flutter build macos --release --target-arch=arm64
```

This produces `build/macos/Build/Products/Release/rdcs_client.app`.

### Intel Mac (x86_64)

Build on an Intel Mac:

```bash
cd client/flutter
flutter build macos --release --target-arch=x64
```

**Note:** Cross-compilation is not supported. You must build on the target architecture, or rely on the CI/CD pipeline to produce both binaries.

## DMG Packaging

After a successful release build, create a DMG installer for distribution:

```bash
# For the architecture you just built
./scripts/package_macos.sh arm64
# or
./scripts/package_macos.sh x64
```

The script:
- Embeds the built app into a DMG template
- Adds code signing if credentials are available
- Names the file with the version and architecture: `rdcs_client-arm64-vX.X.X.dmg`

## Verifying Architecture

To confirm the correct architecture was built, inspect the binary:

```bash
file client/flutter/build/macos/Build/Products/Release/rdcs_client.app/Contents/Frameworks/librdcs_core.dylib
```

Expected outputs:
- **Apple Silicon (arm64):** `Mach-O 64-bit dynamically linked shared library arm64`
- **Intel (x86_64):** `Mach-O 64-bit dynamically linked shared library x86_64`

You can also use `lipo` to check which architectures are in a fat binary (if universal binaries are used in future):

```bash
lipo -info client/flutter/build/macos/Build/Products/Release/rdcs_client.app/Contents/Frameworks/librdcs_core.dylib
```

## CI/CD Pipeline

The GitHub Actions workflow automatically builds release packages for both architectures on every tag push or manual dispatch:

1. **Checkout** the repository
2. **Setup** Flutter, Rust, and build tools
3. **Install Rust targets** for both arm64 and x86_64
4. **Build** the Flutter app for each architecture
5. **Package** each into a DMG
6. **Upload** artifacts and create release assets

### Automated Releases

When you push a tag (e.g., `git tag v1.0.0 && git push --tags`), the CI pipeline automatically:
- Builds both `rdcs_client-arm64-vX.X.X.dmg` and `rdcs_client-x86_64-vX.X.X.dmg`
- Creates a GitHub release
- Attaches both DMGs to the release

Users can then download the appropriate DMG for their hardware.

## Dependencies and Requirements

### macOS Development Environment

- **macOS:** 11.0+ (Apple Silicon) or 10.15+ (Intel)
- **Xcode:** 13 or later (includes clang, linker, SDKs)
- **Rust:** 1.70 or later
- **Flutter:** 3.x

Verify your setup:

```bash
flutter doctor -v
rustc --version
xcode-select --print-path
```

### Rust Targets

The build scripts automatically install the required Rust compilation targets:
- `aarch64-apple-darwin` (Apple Silicon)
- `x86_64-apple-darwin` (Intel)

To manually install:

```bash
rustup target add aarch64-apple-darwin
rustup target add x86_64-apple-darwin
```

List installed targets:

```bash
rustup target list | grep darwin
```

### macOS SDKs

Xcode includes the necessary SDKs. If SDKs are missing, reinstall Xcode command-line tools:

```bash
xcode-select --install
# or reset to default
xcode-select --reset
```

## Build Performance

Typical build times on modern hardware:

- **Debug build (current arch):** 2–3 minutes
- **Release build (current arch):** 5–8 minutes
- **DMG packaging:** 30–60 seconds

Times vary based on CPU, disk speed, and whether dependencies are cached.

## Common Questions

### Q: Can I build the Intel version on Apple Silicon?

**A:** No. Direct cross-compilation for macOS is not supported in Flutter's build system. You must either:
- Build on an Intel Mac
- Use the CI/CD pipeline to produce both binaries
- Use virtualization (slower, not recommended for release builds)

### Q: Can I build the Apple Silicon version on Intel?

**A:** No, for the same reason. The CI/CD pipeline is the best solution for building both architectures.

### Q: How do I know which architecture my Mac is?

**A:** Run:

```bash
uname -m
```

- **arm64:** Apple Silicon
- **x86_64:** Intel

Or check System Preferences > About > Chip.

### Q: What if the build fails with "target not found"?

**A:** Ensure the Rust target is installed:

```bash
rustup target add aarch64-apple-darwin
rustup target add x86_64-apple-darwin
```

Then clean and rebuild:

```bash
flutter clean
flutter build macos --release --target-arch=arm64
```

### Q: Can I distribute both architectures in a single app bundle?

**A:** Not with the current setup. Each architecture is built and packaged separately. This approach:
- Keeps binaries small (~50–60 MB per arch)
- Avoids code signing complexity
- Simplifies testing and debugging

If universal binaries are needed in the future, the build scripts can be updated to use `lipo` to merge architectures.

### Q: How large is the DMG?

**A:** Typically 50–70 MB per architecture, depending on dependencies and build configuration.

### Q: What if I want to sign and notarize the app?

**A:** Code signing and notarization are optional for development. For production distribution:

1. Obtain an Apple Developer certificate
2. Set environment variables for signing identity and team ID
3. The CI/CD pipeline or build scripts can be updated to integrate `codesign` and `xcrun notarytool`

See Apple's documentation on [Notarizing macOS Software Before Distribution](https://developer.apple.com/documentation/xcode/notarizing_macos_software_before_distribution).

### Q: Can I use a different compiler or linker?

**A:** The build system uses the default Xcode toolchain. To use a different setup:
- Modify `~/.cargo/config.toml` for Rust-specific overrides
- Modify the Flutter/Dart build configuration in `pubspec.yaml`

However, this is not recommended unless you have specific performance or compatibility requirements.

## Troubleshooting

### Build fails with "could not compile dependencies"

Clean and retry:

```bash
flutter clean
cargo clean
flutter pub get
flutter build macos --release
```

### Architecture mismatch errors

Verify the target architecture matches your hardware or CI environment:

```bash
# Show current machine architecture
uname -m

# Show target architectures in build output
flutter build macos --release --verbose 2>&1 | grep -i arch
```

### Code signing errors on CI

Ensure the correct code-signing identity is available:

```bash
security find-identity -v -p codesigning
```

If using CI, ensure the signing certificate and provisioning profile are configured in CI secrets.

### Flutter/Rust version conflicts

Update both toolchains:

```bash
flutter upgrade
rustup update
```

Then regenerate bindings:

```bash
cd client/flutter
flutter pub get
cargo build
```

## Next Steps

- For multi-architecture deployment strategies, see [Deployment Guide](./DEPLOYMENT.md) (if available)
- For CI/CD configuration details, see `.github/workflows/`
- For Rust FFI and bindings, see `crates/rdcs-ffi/`
