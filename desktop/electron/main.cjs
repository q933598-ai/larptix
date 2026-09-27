const { app, BrowserWindow, desktopCapturer, dialog, session, shell } = require("electron");

const serverUrl = new URL(process.env.LARPTRIX_SERVER_URL || "http://127.0.0.1:8080");

if (!new Set(["http:", "https:"]).has(serverUrl.protocol)) {
  throw new Error("LARPTRIX_SERVER_URL must use HTTP or HTTPS");
}

function isServerOrigin(rawUrl) {
  try {
    return new URL(rawUrl).origin === serverUrl.origin;
  } catch {
    return false;
  }
}

async function createMainWindow() {
  const defaultSession = session.defaultSession;
  defaultSession.setPermissionCheckHandler((_webContents, permission, origin) => {
    return ["media", "display-capture"].includes(permission) && isServerOrigin(origin);
  });
  defaultSession.setPermissionRequestHandler((webContents, permission, callback, details) => {
    const requestingUrl = details.requestingUrl || webContents.getURL();
    callback(["media", "display-capture"].includes(permission) && isServerOrigin(requestingUrl));
  });

  const window = new BrowserWindow({
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
    },
  });

  defaultSession.setDisplayMediaRequestHandler(async (request, callback) => {
    if (!isServerOrigin(request.securityOrigin) || !request.videoRequested) {
      callback({});
      return;
    }

    try {
      const sources = await desktopCapturer.getSources({
        types: ["screen", "window"],
        thumbnailSize: { width: 320, height: 180 },
        fetchWindowIcons: true,
      });
      if (!sources.length) {
        callback({});
        return;
      }
      const buttons = sources.map((source) => source.name).slice(0, 8);
      buttons.push("Cancel");
      const cancelId = buttons.length - 1;
      const choice = await dialog.showMessageBox(window, {
        type: "question",
        title: "Share your screen",
        message: "Choose a screen or window to share",
        buttons,
        cancelId,
        defaultId: 0,
        noLink: true,
      });
      if (choice.response === cancelId || choice.response >= sources.length) {
        callback({});
        return;
      }

      const source = sources[choice.response];
      callback({ video: source });
    } catch (error) {
      console.error("Screen capture picker failed:", error);
      callback({});
    }
  });

  window.webContents.setWindowOpenHandler(({ url }) => {
    if (isServerOrigin(url)) {
      void window.loadURL(url);
    } else {
      void shell.openExternal(url);
    }
    return { action: "deny" };
  });
  window.webContents.on("will-navigate", (event, url) => {
    if (!isServerOrigin(url)) {
      event.preventDefault();
      void shell.openExternal(url);
    }
  });

  await window.loadURL(serverUrl.toString());
}

app.whenReady().then(createMainWindow);

app.on("activate", () => {
  if (BrowserWindow.getAllWindows().length === 0) {
    void createMainWindow();
  }
});

app.on("window-all-closed", () => {
  if (process.platform !== "darwin") app.quit();
});