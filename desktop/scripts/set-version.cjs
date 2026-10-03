const fs = require("fs");
const path = require("path");

const version = process.argv[2];

if (!/^\d+\.\d+\.\d+$/.test(version || "")) {
  throw new Error("Usage: npm run version:set -- X.Y.Z");
}

const desktopRoot = path.join(__dirname, "..");
const packagePath = path.join(desktopRoot, "package.json");
const lockPath = path.join(desktopRoot, "package-lock.json");
const tauriPath = path.join(desktopRoot, "src-tauri", "tauri.conf.json");
const cargoPath = path.join(desktopRoot, "src-tauri", "Cargo.toml");

const packageJson = JSON.parse(fs.readFileSync(packagePath, "utf8"));
packageJson.version = version;
fs.writeFileSync(packagePath, JSON.stringify(packageJson, null, 2) + "\n");

const lock = JSON.parse(fs.readFileSync(lockPath, "utf8"));
lock.version = version;
if (lock.packages?.[""]) {
  lock.packages[""].version = version;
}
fs.writeFileSync(lockPath, JSON.stringify(lock, null, 2) + "\n");

const tauri = JSON.parse(fs.readFileSync(tauriPath, "utf8"));
tauri.version = version;
const [major, minor, patch] = version.split(".").map(Number);
tauri.bundle ??= {};
tauri.bundle.android ??= {};
tauri.bundle.android.versionCode =
  1_000_000 + major * 100_000 + minor * 1_000 + patch;
fs.writeFileSync(tauriPath, JSON.stringify(tauri, null, 2) + "\n");

let cargo = fs.readFileSync(cargoPath, "utf8");
cargo = cargo.replace(/^(version\s*=\s*")[^"]+(")$/m, `$1${version}$2`);
fs.writeFileSync(cargoPath, cargo);

console.log(`Larptrix version set to ${version} (Android versionCode ${tauri.bundle.android.versionCode}).`);
