set dotenv-required := true
set dotenv-filename := ".env"

no_pattern := ""

# List all available commands
default:
    just --list

# ==============================================================================
# Onboarding
# ==============================================================================

# Install other dependencies used during development
install-system-dependencies:
    # https://lib.rs/crates/cargo-llvm-cov
    cargo +stable install cargo-llvm-cov
    # https://github.com/kbknapp/cargo-outdated
    cargo +stable install cargo-outdated

# ==============================================================================
# Lint
# ==============================================================================

lint:
    cargo fmt -- --check
    cargo clippy --tests --verbose -- -D warnings
    cargo outdated --root-deps-only

# ==============================================================================
# GitHub Actions
# ==============================================================================

# Run the release github action locally
run-release-action:
    act -W .github/workflows/release.yml workflow_dispatch

# Run the CI github action locally
run-ci-action:
    act -W .github/workflows/ci.yml pull_request rust/main

# ==============================================================================
# Test
# ==============================================================================

# Run all tests and generate a coverage report
coverage:
    cargo llvm-cov --html --open

# Run all tests
test:
    cargo test -- --nocapture --color=always

# Run test in watch mode
watch pattern=no_pattern:
    cargo watch --clear --exec 'test -- --nocapture --color=always {{pattern}}'

# Run the rust binary against an unformatted test fixture
run-misc:
    cd fixtures/misc
    RUST_BACKTRACE=1 cargo run -- lint --source 'package.json'

# Run the dev rust binary against a clone of microsoft/FluidFramework
run-fluid:
    cd fixtures/fluid-framework
    RUST_BACKTRACE=1 cargo run -- lint --versions

# Run the release rust binary against a clone of microsoft/FluidFramework
run-fluid-prod:
    cd fixtures/fluid-framework
    ../../target/release/syncpack lint --versions --source 'package.json' --source 'packages/**/package.json'

# ==============================================================================
# Build
# ==============================================================================

# Build a rust binary and corresponding npm package for a specific target
build-binary-package:
    just create-rust-binary
    just create-npm-binary-package

# Build a rust binary for a specific target
create-rust-binary:
    #!/usr/bin/env bash
    set -euxo pipefail

    cargo build --release --locked --target "$TARGET"

# Once a rust binary for a specific target has been built, create an npm package for it
create-npm-binary-package:
    #!/usr/bin/env bash
    set -euxo pipefail

    rm -rf "$NODE_PKG_DIR_PATH"
    mkdir -p "$NODE_PKG_DIR_PATH/bin"
    mv "$RUST_BINARY_PATH" "$NODE_PKG_RUST_BINARY_PATH"
    cp README.md "$NODE_PKG_DIR_PATH/README.md"
    just create-npm-binary-package-json

# Create the package.json file for an npm package for a specific target
create-npm-binary-package-json:
    #!/usr/bin/env node
    const fs = require("fs");
    const path = require("path");
    const srcPath = path.resolve("package.json");
    const destPath = path.resolve(process.env.NODE_PKG_DIR_PATH, "package.json");
    const pkg = require(srcPath);
    const nextPkg = {
        ...pkg,
        name: process.env.NODE_PKG_NAME,
        bin: undefined,
        optionalDependencies: undefined,
        os: [process.env.NODE_OS],
        cpu: [process.env.NODE_ARCH],
    };
    const json = JSON.stringify(nextPkg, null, 2);
    console.log(json);
    fs.writeFileSync(destPath, json);

# Create the parent npm package which delegates to each target-specific package
create-npm-root-package:
    #!/usr/bin/env bash
    set -euxo pipefail

    rm -rf "$NODE_ROOT_PKG_DIR_PATH"
    mkdir -p "$NODE_ROOT_PKG_DIR_PATH"
    cp README.md "$NODE_ROOT_PKG_DIR_PATH/README.md"
    cp npm/index.js "$NODE_ROOT_PKG_DIR_PATH/index.js"
    just create-npm-root-package-json

# Create the package.json file for the parent npm package
create-npm-root-package-json:
    #!/usr/bin/env node
    const fs = require("fs");
    const path = require("path");
    const srcPath = path.resolve("package.json");
    const destPath = path.resolve(process.env.NODE_ROOT_PKG_DIR_PATH, "package.json");
    const pkg = require(srcPath);
    const nextPkg = {
        ...pkg,
        os: undefined,
        cpu: undefined,
        bin: {
          syncpack: "./index.js",
        },
        optionalDependencies: {
          "syncpack-linux-x64": pkg.version,
          "syncpack-linux-arm64": pkg.version,
          "syncpack-darwin-x64": pkg.version,
          "syncpack-darwin-arm64": pkg.version,
          "syncpack-windows-x64": pkg.version,
          "syncpack-windows-arm64": pkg.version,
        },
    };
    const json = JSON.stringify(nextPkg, null, 2);
    console.log(json);
    fs.writeFileSync(destPath, json);

# ==============================================================================
# Publish
# ==============================================================================

# Publish the npm package for a specific target
publish-npm-binary-package:
    #!/usr/bin/env bash
    set -euxo pipefail

    cd "$NODE_PKG_DIR_PATH"
    npm publish --dry-run --access public --tag rust

# Publish the parent npm package
publish-npm-root-package:
    #!/usr/bin/env bash
    set -euxo pipefail

    cd "$NODE_ROOT_PKG_DIR_PATH"
    npm publish --dry-run --access public --tag rust
