import path from "path";
import fs from "fs";

const __dirname = import.meta.dirname;
const root = path.resolve(__dirname, "../..");
const pkgSrc = path.resolve(root, "package.json");
const pkgDest = path.resolve(root, "npm/root/package.json");
const pkg = JSON.parse(fs.readFileSync(pkgSrc, "utf8"));

const json = JSON.stringify(
  {
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
  },
  null,
  2,
);

console.log(json);

fs.writeFileSync(pkgDest, json);
