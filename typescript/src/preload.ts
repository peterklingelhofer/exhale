// All of the Node.js APIs are available in the preload process.
// It has the same sandbox as a Chrome extension.

import { contextBridge, ipcRenderer } from "electron";

// Expose store API to renderer process
contextBridge.exposeInMainWorld("store", {
  get: (key: string) => ipcRenderer.invoke("store-get", key),
  set: (key: string, value: unknown) => ipcRenderer.invoke("store-set", key, value),
});
