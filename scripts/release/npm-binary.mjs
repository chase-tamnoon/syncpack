import path from "path";
import fs from "fs";

const [_node, _program, nodeArch, nodeOs, nodePkgDirPath, nodePkgName] =
  process.argv;

const __dirname = import.meta.dirname;
const root = path.resolve(__dirname, "../..");
const pkgSrc = path.resolve(root, "package.json");
const pkgDest = path.resolve(nodePkgDirPath, "package.json");
const pkg = JSON.parse(fs.readFileSync(pkgSrc, "utf8"));

const json = JSON.stringify(
  {
    ...pkg,
    name: nodePkgName,
    bin: undefined,
    optionalDependencies: undefined,
    os: [nodeOs],
    cpu: [nodeArch],
  },
  null,
  2,
);

console.log(json);

fs.writeFileSync(pkgDest, json);
