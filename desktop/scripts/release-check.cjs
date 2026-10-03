const fs = require("fs");
const path = require("path");

const desktopRoot = path.join(__dirname, "..");

const packageJson = JSON.parse(
  fs.readFileSync(path.join(desktopRoot, "package.json"), "utf8"),
);
const tauriConfig = JSON.parse(
  fs.readFileSync(path.join(desktopRoot, "src-tauri", "tauri.conf.json"), "utf8"),
);
const cargoToml = fs.readFileSync(
  path.join(desktopRoot, "src-tauri", "Cargo.toml"),
  "utf8",
);

const expectedTag = process.env.GITHUB_REF_NAME;
const expectedVersion = expectedTag
  ? expectedTag.replace(/^v/i, "")
  : packageJson.version;

if (!/^\d+\.\d+\.\d+$/.test(packageJson.version)) {
  throw new Error(`Invalid package version: ${packageJson.version}`);
}

for (const [name, version] of [
  ["package.json", packageJson.version],
  ["tauri.conf.json", tauriConfig.version],
  ["Cargo.toml", cargoToml.match(/^version\s*=\s*"([^"]+)"/m)?.[1]],
]) {
  if (version !== expectedVersion) {
    throw new Error(
      `Release version mismatch: tag=${expectedVersion}, ${name}=${version || "missing"}`,
    );
  }
}

const [major, minor, patch] = expectedVersion.split(".").map(Number);
const expectedVersionCode = major * 1_000_000 + minor * 1_000 + patch;
const actualVersionCode = Number(tauriConfig.bundle?.android?.versionCode);

if (!Number.isInteger(actualVersionCode) || actualVersionCode !== expectedVersionCode) {
  throw new Error(
    `Android versionCode ${actualVersionCode} is too low; expected at least ${expectedVersionCode}`,
  );
}

console.log(`Larptrix release ${expectedVersion} is valid (Android versionCode ${actualVersionCode}).`);
