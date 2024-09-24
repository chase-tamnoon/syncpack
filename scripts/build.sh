#!/usr/bin/env bash

echo "RUST_BINARY_PATH=target/${{ matrix.build.TARGET }}/release/syncpack" >> $GITHUB_OUTPUT
echo "NODE_PKG_DIR_PATH=npm/binaries/${{ matrix.build.NODE_PKG_NAME }}" >> $GITHUB_OUTPUT

cargo build --release --locked --target ${{ matrix.build.TARGET }}

# NPM ENV
node_arch="${{ matrix.build.NODE_ARCH }}"
node_os="${{ matrix.build.NODE_OS }}"
node_pkg_dir_path="${{ steps.config.outputs.NODE_PKG_DIR_PATH }}"
node_pkg_name="${{ matrix.build.NODE_PKG_NAME }}"
rust_binary_path="${{ steps.config.outputs.RUST_BINARY_PATH }}"
# CREATE /BIN DIRECTORY
mkdir -p "${node_pkg_dir_path}/bin"
# COPY RUST BINARY
cp "${rust_binary_path}" "${node_pkg_dir_path}/bin/syncpack"
# COPY README.MD
cp README.md "${node_pkg_dir_path}/README.md"
# WRITE PACKAGE.JSON
node scripts/release/npm-binary.mjs \
  "${node_arch}" \
  "${node_os}" \
  "${node_pkg_dir_path}" \
  "${node_pkg_name}"
# PUBLISH BINARY PACKAGE TO NPM
cd "${node_pkg_dir_path}"
npm publish --dry-run --access public --tag rust


# NPM BASE
# COPY README.MD
cp README.md npm/root/README.md
# WRITE PACKAGE.JSON
node scripts/release/npm-root.mjs
# PUBLISH ROOT PACKAGE TO NPM
cd npm/root
npm publish --dry-run --access public --tag rust
