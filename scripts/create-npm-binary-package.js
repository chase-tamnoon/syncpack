const path = require("path");
const fs = require("fs");

const NODE_ARCH = process.env.NODE_ARCH
const NODE_OS = process.env.NODE_OS
const NODE_PKG_NAME = process.env.NODE_PKG_NAME

const root = process.cwd();
const pkgSrc = path.resolve(root, "package.json");
const pkgDest = path.resolve(root, "npm/binaries", NODE_PKG_NAME, "package.json");
const pkg = JSON.parse(fs.readFileSync(pkgSrc, "utf8"));
const json = JSON.stringify(
  {
    ...pkg,
    name: NODE_PKG_NAME,
    bin: undefined,
    optionalDependencies: undefined,
    os: [NODE_OS],
    cpu: [NODE_ARCH],
  },
  null,
  2,
);

console.log({
  root,
  pkgSrc,
  pkgDest,
  pkg,
  json,
});

// fs.writeFileSync(pkgDest, json);
