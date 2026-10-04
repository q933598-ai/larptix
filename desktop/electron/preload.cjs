const { contextBridge, ipcRenderer } = require("electron");

contextBridge.exposeInMainWorld("larptrixDesktop", {
  isDesktop: true,
  platform: process.platform,
  arch: process.arch,
  version: process.versions.electron,
  appVersion: () => ipcRenderer.invoke("larptrix:app-version"),
  checkForUpdate: () => ipcRenderer.invoke("larptrix:update-check"),
  installUpdate: () => ipcRenderer.invoke("larptrix:update-install"),
  toggleFullscreen: () => ipcRenderer.invoke("larptrix:toggle-fullscreen"),
  openReleases: () => ipcRenderer.invoke("larptrix:open-releases"),
});
