const {
  app,
  BrowserWindow,
  desktopCapturer,
  dialog,
  ipcMain,
  session,
  shell,
} = require("electron");
const crypto = require("crypto");
const fs = require("fs");
const path = require("path");
const { spawn } = require("child_process");

const PERSISTENT_PARTITION = "persist:larptrix";
const UPDATE_REPOSITORY = "q933598-ai/larptix";
const UPDATE_API_URL = `https://api.github.com/repos/${UPDATE_REPOSITORY}/releases/latest`;

let serverUrl = null;
let windowRef = null;
let updateDialogOpen = false;

function parseServerUrl(rawUrl) {
  try {
    const url = new URL(rawUrl.trim());

    if (!["http:", "https:"].includes(url.protocol)) {
      return null;
    }

    if (!url.hostname || url.username || url.password) {
      return null;
    }

    return url;
  } catch {
    return null;
  }
}

function isServerOrigin(rawUrl) {
  if (!serverUrl) {
    return false;
  }

  try {
    return new URL(rawUrl).origin === serverUrl.origin;
  } catch {
    return false;
  }
}

function isBootstrapUrl(rawUrl) {
  try {
    const target = new URL(rawUrl);

    if (target.protocol !== "file:") {
      return false;
    }

    return (
      path.resolve(decodeURIComponent(target.pathname)) ===
      path.resolve(path.join(__dirname, "..", "bootstrap", "index.html"))
    );
  } catch {
    return false;
  }
}

function loadBootstrap() {
  return windowRef.loadFile(
    path.join(__dirname, "..", "bootstrap", "index.html"),
  );
}

function normalizeVersion(value) {
  const match = String(value || "").trim().replace(/^v/i, "").match(/^(\d+)(?:\.(\d+))?(?:\.(\d+))?/);
  if (!match) return null;
  return [Number(match[1]), Number(match[2] || 0), Number(match[3] || 0)];
}

function compareVersions(left, right) {
  const a = normalizeVersion(left);
  const b = normalizeVersion(right);
  if (!a || !b) return 0;

  for (let i = 0; i < 3; i += 1) {
    if (a[i] > b[i]) return 1;
    if (a[i] < b[i]) return -1;
  }
  return 0;
}

async function fetchLatestRelease() {
  const response = await fetch(UPDATE_API_URL, {
    headers: {
      Accept: "application/vnd.github+json",
      "User-Agent": `Larptrix/${app.getVersion()}`,
    },
  });

  if (!response.ok) {
    throw new Error(`GitHub Releases returned HTTP ${response.status}`);
  }

  return response.json();
}

function preferredUpdateAsset(release) {
  const version = release?.tag_name?.replace(/^v/i, "");
  if (!version || !Array.isArray(release.assets)) return null;

  if (process.platform === "win32") {
    return release.assets.find((asset) =>
      asset.name === `Larptrix-${version}-win-x64.exe`
    ) || null;
  }

  if (process.platform === "linux") {
    return release.assets.find((asset) =>
      asset.name === `Larptrix-${version}-x64.pacman`
      || asset.name === `Larptrix-${version}-x64.pkg.tar.zst`
    ) || null;
  }

  return null;
}

async function getAvailableDesktopUpdate() {
  if (!app.isPackaged) return null;

  const release = await fetchLatestRelease();
  const latestVersion = release.tag_name;
  if (compareVersions(latestVersion, app.getVersion()) <= 0) {
    return null;
  }

  const asset = preferredUpdateAsset(release);
  if (!asset?.browser_download_url) {
    return null;
  }

  const checksums = release.assets.find((candidate) => candidate.name === "SHA256SUMS.txt");

  return {
    version: latestVersion.replace(/^v/i, ""),
    currentVersion: app.getVersion(),
    notes: typeof release.body === "string" ? release.body.slice(0, 4000) : "",
    assetName: asset.name,
    assetUrl: asset.browser_download_url,
    checksumsUrl: checksums?.browser_download_url || null,
    releaseUrl: release.html_url,
  };
}

async function downloadFile(url, destination) {
  const response = await fetch(url, {
    headers: {
      Accept: "application/octet-stream",
      "User-Agent": `Larptrix/${app.getVersion()}`,
    },
  });

  if (!response.ok || !response.body) {
    throw new Error(`Could not download update (HTTP ${response.status})`);
  }

  const handle = await fs.promises.open(destination, "w");
  try {
    for await (const chunk of response.body) {
      await handle.write(chunk);
    }
  } finally {
    await handle.close();
  }
}

async function verifyDownloadedFile(filePath, assetName, checksumsUrl) {
  if (!checksumsUrl) return;

  const response = await fetch(checksumsUrl, {
    headers: {
      Accept: "text/plain",
      "User-Agent": `Larptrix/${app.getVersion()}`,
    },
  });
  if (!response.ok) {
    throw new Error("Could not download update checksums.");
  }

  const checksumText = await response.text();
  const expectedLine = checksumText
    .split(/\r?\n/)
    .find((line) => line.trim().endsWith(`  ${assetName}`));

  if (!expectedLine) {
    throw new Error("Update checksum is missing.");
  }

  const expected = expectedLine.trim().split(/\s+/)[0].toLowerCase();
  if (!/^[a-f0-9]{64}$/.test(expected)) {
    throw new Error("Update checksum is invalid.");
  }

  const hash = crypto.createHash("sha256");
  const stream = fs.createReadStream(filePath);

  await new Promise((resolve, reject) => {
    stream.on("data", (chunk) => hash.update(chunk));
    stream.on("end", resolve);
    stream.on("error", reject);
  });

  const actual = hash.digest("hex").toLowerCase();
  if (actual !== expected) {
    throw new Error("Update checksum verification failed.");
  }
}

function runElevated(command, args) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, {
      detached: false,
      stdio: "inherit",
    });

    child.once("error", reject);
    child.once("exit", (code, signal) => {
      if (code === 0) {
        resolve();
      } else {
        reject(new Error(
          signal
            ? `Updater stopped by signal ${signal}.`
            : `Updater exited with code ${code}.`,
        ));
      }
    });
  });
}

async function installDesktopUpdate(update) {
  const tempName = update.assetName.replace(/[^a-zA-Z0-9._-]/g, "_");
  const installerPath = path.join(app.getPath("temp"), tempName);

  try {
    await downloadFile(update.assetUrl, installerPath);
    await verifyDownloadedFile(installerPath, update.assetName, update.checksumsUrl);

    if (process.platform === "win32") {
      const installer = spawn(installerPath, ["/S"], {
        detached: true,
        stdio: "ignore",
        windowsHide: false,
      });
      installer.unref();
      setTimeout(() => app.quit(), 800);
      return;
    }

    if (process.platform === "linux") {
      const usePkexec = fs.existsSync("/usr/bin/pkexec");
      const command = usePkexec ? "pkexec" : "sudo";
      const args = usePkexec
        ? ["pacman", "-U", "--noconfirm", installerPath]
        : ["pacman", "-U", "--noconfirm", installerPath];

      await runElevated(command, args);
      app.quit();
      return;
    }

    throw new Error("Automatic desktop updates are not supported on this platform.");
  } finally {
    if (process.platform !== "win32") {
      void fs.promises.rm(installerPath, { force: true }).catch(() => {});
    }
  }
}

async function checkAndOfferDesktopUpdate() {
  if (!app.isPackaged || updateDialogOpen || !windowRef || windowRef.isDestroyed()) {
    return;
  }

  try {
    const update = await getAvailableDesktopUpdate();
    if (!update) return;

    updateDialogOpen = true;
    try {
      const result = await dialog.showMessageBox(windowRef, {
        type: "info",
        title: "Larptrix update available",
        message: `Larptrix ${update.version} is available.`,
        detail: `Current version: ${update.currentVersion}\n\n${update.notes || "A newer version is ready to install."}`,
        buttons: ["Update now", "Later"],
        defaultId: 0,
        cancelId: 1,
        noLink: true,
      });

      if (result.response === 0) {
        try {
          await installDesktopUpdate(update);
        } catch (error) {
          await dialog.showMessageBox(windowRef, {
            type: "error",
            title: "Larptrix update failed",
            message: "The update could not be installed.",
            detail: error?.message || String(error),
            buttons: ["OK"],
          });
        }
      }
    } finally {
      updateDialogOpen = false;
    }
  } catch (error) {
    console.warn("Desktop update check failed:", error?.message || error);
  }
}

function scheduleDesktopUpdates() {
  if (!app.isPackaged) return;

  setTimeout(() => void checkAndOfferDesktopUpdate(), 7000);
  setInterval(() => void checkAndOfferDesktopUpdate(), 6 * 60 * 60 * 1000);
}

async function createMainWindow() {
  // Advanced override:
  // LARPTRIX_SERVER_URL=https://example.com npm run dev
  const envServer = process.env.LARPTRIX_SERVER_URL;

  if (envServer) {
    serverUrl = parseServerUrl(envServer);

    if (!serverUrl) {
      throw new Error(
        "LARPTRIX_SERVER_URL must contain a valid HTTP(S) URL",
      );
    }
  } else {
    serverUrl = null;
  }

  // Persistent Electron storage.
  // Cookies, localStorage and IndexedDB survive application restarts.
  const persistentSession = session.fromPartition(
    PERSISTENT_PARTITION,
  );

  persistentSession.setPermissionCheckHandler(
    (_webContents, permission, origin) => {
      return (
        (["media", "display-capture"].includes(permission) ||
          permission === "notifications") &&
        isServerOrigin(origin)
      );
    },
  );

  persistentSession.setPermissionRequestHandler(
    (webContents, permission, callback, details) => {
      const requestingUrl =
        details.requestingUrl || webContents.getURL();

      callback(
        (["media", "display-capture"].includes(permission) ||
          permission === "notifications") &&
          isServerOrigin(requestingUrl),
      );
    },
  );

  windowRef = new BrowserWindow({
    width: 1200,
    height: 800,
    minWidth: 800,
    minHeight: 560,
    title: "Larptrix",
    autoHideMenuBar: true,

    webPreferences: {
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: true,
      backgroundThrottling: false,

      // Keep authentication and client storage between launches.
      partition: PERSISTENT_PARTITION,
      preload: path.join(__dirname, "preload.cjs"),
    },
  });

  persistentSession.setDisplayMediaRequestHandler(
    async (request, callback) => {
      if (
        !isServerOrigin(request.securityOrigin) ||
        !request.videoRequested
      ) {
        callback({});
        return;
      }

      try {
        const sources = await desktopCapturer.getSources({
          types: ["screen", "window"],
          thumbnailSize: {
            width: 320,
            height: 180,
          },
          fetchWindowIcons: true,
        });

        if (!sources.length) {
          callback({});
          return;
        }

        const buttons = sources
          .map((source) => source.name)
          .slice(0, 8);

        buttons.push("Cancel");

        const cancelId = buttons.length - 1;

        const choice = await dialog.showMessageBox(windowRef, {
          type: "question",
          title: "Share your screen",
          message: "Choose a screen or window to share",
          buttons,
          cancelId,
          defaultId: 0,
          noLink: true,
        });

        if (
          choice.response === cancelId ||
          choice.response >= sources.length
        ) {
          callback({});
          return;
        }

        callback({
          video: sources[choice.response],
          ...(process.platform === "win32" && request.audioRequested
            ? { audio: "loopback" }
            : {}),
        });
      } catch (error) {
        console.error(
          "Screen capture picker failed:",
          error,
        );
        callback({});
      }
    },
  );

  windowRef.webContents.setWindowOpenHandler(({ url }) => {
    if (isServerOrigin(url)) {
      void windowRef.loadURL(url);
    } else {
      void shell.openExternal(url);
    }

    return { action: "deny" };
  });

  windowRef.webContents.on(
    "will-navigate",
    (event, url) => {
      const currentUrl = windowRef.webContents.getURL();

      // Bootstrap is allowed to navigate to the server
      // selected by the user.
      if (isBootstrapUrl(currentUrl)) {
        const selectedServer = parseServerUrl(url);

        if (selectedServer) {
          serverUrl = selectedServer;
          return;
        }
      }

      // Once connected, only the selected Larptix server
      // may navigate inside the application.
      if (isServerOrigin(url)) {
        return;
      }

      event.preventDefault();
      void shell.openExternal(url);
    },
  );

  if (serverUrl) {
    await windowRef.loadURL(serverUrl.toString());
  } else {
    await loadBootstrap();
  }

  scheduleDesktopUpdates();
}

ipcMain.handle("larptrix:app-version", () => app.getVersion());
ipcMain.handle("larptrix:open-releases", async () => {
  await shell.openExternal(`https://github.com/${UPDATE_REPOSITORY}/releases/latest`);
  return true;
});
ipcMain.handle("larptrix:update-check", async () => getAvailableDesktopUpdate());
ipcMain.handle("larptrix:update-install", async () => {
  const update = await getAvailableDesktopUpdate();
  if (!update) return { installed: false, reason: "no-update" };
  await installDesktopUpdate(update);
  return { installed: true };
});

app.whenReady().then(createMainWindow);

app.on("activate", () => {
  if (BrowserWindow.getAllWindows().length === 0) {
    void createMainWindow();
  }
});

app.on("window-all-closed", () => {
  if (process.platform !== "darwin") {
    app.quit();
  }
});
