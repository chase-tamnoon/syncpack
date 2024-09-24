set dotenv-required := true
set dotenv-filename := ".env"

# List all available commands
default:
    just --list

# Build a Rust binary and corresponding NPM package for a specific target
build-binary-package:
    just create-rust-binary
    just create-npm-binary-package

# Build a Rust binary for a specific target
create-rust-binary:
    #!/usr/bin/env bash
    set -euxo pipefail

    cargo build --release --locked --target "$TARGET"

# Once a Rust binary for a specific target has been built, create an NPM package for it
create-npm-binary-package:
    #!/usr/bin/env bash
    set -euxo pipefail

    rm -rf "$NODE_PKG_DIR_PATH"
    mkdir -p "$NODE_PKG_DIR_PATH/bin"
    mv "$RUST_BINARY_PATH" "$NODE_PKG_RUST_BINARY_PATH"
    cp README.md "$NODE_PKG_DIR_PATH/README.md"
    just create-npm-binary-package-json

# Create the package.json file for an NPM package for a specific target
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

# Create the parent NPM package which delegates to each target-specific package
create-npm-root-package:
    #!/usr/bin/env bash
    set -euxo pipefail

    rm -rf "$NODE_ROOT_PKG_DIR_PATH"
    mkdir -p "$NODE_ROOT_PKG_DIR_PATH"
    cp README.md "$NODE_ROOT_PKG_DIR_PATH/README.md"
    cp npm/index.js "$NODE_ROOT_PKG_DIR_PATH/index.js"
    just create-npm-root-package-json

# Create the package.json file for the parent NPM package
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

# Publish the NPM package for a specific target
publish-npm-binary-package:
    #!/usr/bin/env bash
    set -euxo pipefail

    cd "$NODE_PKG_DIR_PATH"
    npm publish --dry-run --access public --tag rust

# Publish the parent NPM package
publish-npm-root-package:
    #!/usr/bin/env bash
    set -euxo pipefail

    cd "$NODE_ROOT_PKG_DIR_PATH"
    npm publish --dry-run --access public --tag rust
