# Larptrix releases

Larptrix releases are created from semantic-version Git tags such as `v0.4.1`.

## Create a release

From `desktop/`:

```bash
npm run version:set -- 0.4.1
git add package.json package-lock.json src-tauri/tauri.conf.json src-tauri/Cargo.toml
git commit -m "release: 0.4.1"
git push origin main
git tag v0.4.1
git push origin v0.4.1
```

The `Release Larptrix` workflow then builds Linux AppImage + pacman, a Windows NSIS installer, and a signed Android APK, creates `SHA256SUMS.txt`, and publishes one GitHub Release containing all assets.

## Linux

The Arch package is named `larptrix`.

First install:

```bash
sudo pacman -U ./Larptrix-0.4.1-x64.pacman
```

After that Larptrix is managed as a pacman package. The in-app updater downloads the new pacman package and runs `pkexec pacman -U` (or `sudo pacman -U` as a fallback).

The pacman target is intended for Arch-based systems and is currently marked beta by electron-builder, so it should be tested on the supported Arch variants before wide distribution.

## Android signing

Production Android releases must use the same private keystore for every release. Do not commit the keystore or `keystore.properties`.

Create a keystore once:

```bash
keytool -genkey -v -keystore ~/larptrix-release.jks -keyalg RSA -keysize 2048 -validity 10000 -alias larptrix
```

Then add these GitHub Actions secrets:

- `ANDROID_KEY_BASE64`: `base64 -w0 ~/larptrix-release.jks`
- `ANDROID_KEY_ALIAS`: `larptrix`
- `ANDROID_KEY_PASSWORD`: the keystore/key password

The release workflow uses that keystore to sign the APK. Keep a secure backup of the keystore; losing it prevents publishing compatible updates to an already-installed Android app.

Android does not support silent installation of an APK. The Larptrix client checks for a newer release automatically and can open the new APK for the normal Android installation confirmation.

## Desktop updates

Windows uses a per-user NSIS installer. Linux uses the pacman package. Both desktop builds check the latest GitHub Release automatically and verify the downloaded asset against `SHA256SUMS.txt` before installation.
