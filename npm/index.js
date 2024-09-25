#!/usr/bin/env node

const { spawnSync } = require("child_process");

const args = process.argv.slice(2);
const arch = process.arch;
const [os, extension] = ["win32", "cygwin"].includes(process.platform) ?
  ["windows", ".exe"] : [process.platform, ""];
const optionalDep = `syncpack-${os}-${arch}`
const pkgSpecifier = `${optionalDep}/bin/syncpack${extension}`;
const processResult = spawnSync(getPathToBinary(), args, { stdio: "inherit" });

process.exit(processResult.status ?? 0);

function getPathToBinary() {
  try {
    return require.resolve(pkgSpecifier);
  } catch (e) {
    throw new Error(
      `syncpack can't find its Rust binary it expected to be installed as an optionalDependency called "${optionalDep}", the Rust binary should be at ${pkgSpecifier}`,
    );
  }
}
