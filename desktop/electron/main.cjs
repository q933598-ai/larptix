const {
  app,
  BrowserWindow,
  desktopCapturer,
  dialog,
  session,
  shell,
} = require("electron");
const path = require("path");

const PERSISTENT_PARTITION = "persist:larptrix";

let serverUrl = null;
let windowRef = null;

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
}

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
