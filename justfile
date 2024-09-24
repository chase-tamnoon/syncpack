build TARGET NODE_ARCH NODE_OS NODE_PKG_NAME:
    #!/usr/bin/env sh
    @echo 'Create variables'
    rust_binary_path="target/{{TARGET}}/release/syncpack"
    node_pkg_dir_path="npm/binaries/{{NODE_PKG_NAME}}"
    @echo 'Build rust binary'
    cargo build --release --locked --target {{TARGET}}
    @echo 'Create /bin directory'
    mkdir -p "$node_pkg_dir_path/bin"
    @echo 'Copy rust binary'
    cp "$rust_binary_path" "$node_pkg_dir_path/bin/syncpack"
    @echo 'Copy readme.md'
    cp README.md "$node_pkg_dir_path/README.md"
    @echo 'Write package.json'
    node scripts/release/npm-binary.mjs "{{NODE_ARCH}}" "{{NODE_OS}}" "$node_pkg_dir_path" "{{NODE_PKG_NAME}}"

build-locally:
    just build TARGET=x86_64-apple-darwin NODE_ARCH=x64 NODE_OS=darwin NODE_PKG_NAME=syncpack

create-npm-binary-package TARGET NODE_ARCH NODE_OS NODE_PKG_NAME:
    {{TARGET}} {{NODE_ARCH}} {{NODE_OS}} {{NODE_PKG_NAME}} node scripts/create-npm-binary-package.js
