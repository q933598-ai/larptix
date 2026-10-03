const fs = require("fs");
const path = require("path");

const packageJson = JSON.parse(
  fs.readFileSync(path.join(__dirname, "..", "package.json"), "utf8"),
);

const expectedTag = process.env.GITHUB_REF_NAME;
const expectedVersion = expectedTag
  ? expectedTag.replace(/^v/i, "")
  : null;

if (!/^\d+\.\d+\.\d+$/.test(packageJson.version)) {
  throw new Error(`Invalid package version: ${packageJson.version}`);
}

if (expectedVersion && packageJson.version !== expectedVersion) {
  throw new Error(
    `Release tag v${expectedVersion} does not match package.json ${packageJson.version}`,
  );
}

console.log(`Larptrix release ${packageJson.version} is valid.`);
