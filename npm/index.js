#!/usr/bin/env node

const { spawnSync } = require("child_process");

const arch = process.arch;
const [os, extension] = ["win32", "cygwin"].includes(process.platform)
  ? ["windows", ".exe"]
  : [process.platform, ""];
const optionalDep = `syncpack-${os}-${arch}`;
const pkgSpecifier = `${optionalDep}/bin/syncpack${extension}`;

const args = process.argv.slice(2);
const processResult = spawnSync(getPathToBinary(), args, { stdio: "inherit" });

process.exit(processResult.status ?? 0);

function getPathToBinary() {
  try {
    return require.resolve(pkgSpecifier);
  } catch (_) {
    console.error(
      "\x1b[31m%s\n%s\x1b[0m",
      `! syncpack expected an optionalDependency "${optionalDep}" to be installed, containing a Rust binary at "${pkgSpecifier}"`,
      "  Please visit https://github.com/JamieMason/syncpack/issues",
    );
    process.exit(1);
  }
}
