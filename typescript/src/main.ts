// main.ts
import { app, BrowserWindow, ipcMain } from "electron";
import * as path from "path";
import Store from "electron-store";

// Set the app name to ensure config is stored in ~/.config/exhale/
app.setName('exhale');

interface StoreSchema {
  colorExhale: string;
  colorInhale: string;
  colorStyle: string;
  shape: string;
  durationInhale: number;
  durationPostInhalePause: number;
  durationExhale: number;
  durationPostExhalePause: number;
  opacity: number;
}

const defaults: StoreSchema = {
  colorExhale: 'rgb(0, 0, 255)',
  colorInhale: 'rgb(255, 0, 0)',
  colorStyle: 'linear',
  shape: 'fullscreen',
  durationInhale: 5,
  durationPostInhalePause: 0,
  durationExhale: 10,
  durationPostExhalePause: 0,
  opacity: 0.25
};

const store = new Store<StoreSchema>({ defaults });

// Initialize store with defaults if not already set
// This ensures defaults are persisted to disk on first run
Object.entries(defaults).forEach(([key, value]) => {
  if (!store.has(key as keyof StoreSchema)) {
    store.set(key as keyof StoreSchema, value);
  }
});

// Register IPC handlers before creating window
ipcMain.handle("store-get", (event, key) => {
  return store.get(key);
});

ipcMain.handle("store-set", (event, key, value) => {
  store.set(key, value);
});

let mainWindow: BrowserWindow;

function createWindow() {
  // Create the browser window.
  mainWindow = new BrowserWindow({
    alwaysOnTop: true,
    height: 600,
    movable: false,
    resizable: true,
    show: true,
    titleBarStyle: "hidden",
    webPreferences: {
      preload: path.join(import.meta.dirname, "preload.js"),
      nodeIntegration: false,
      contextIsolation: true,
    },
    width: 800,
  });

  mainWindow.loadFile(path.join(import.meta.dirname, "../index.html"));
  mainWindow.setAlwaysOnTop(true, "floating", 1);
  mainWindow.setVisibleOnAllWorkspaces(true, { visibleOnFullScreen: true });
  mainWindow.setFullScreenable(false);
  mainWindow.setFocusable(true);
  mainWindow.setIgnoreMouseEvents(true);
  mainWindow.removeMenu();
  mainWindow.webContents.on("devtools-focused", () => {
    mainWindow.setOpacity(1.0);
    mainWindow.setIgnoreMouseEvents(false);
  });
}

// This method will be called when Electron has finished
// initialization and is ready to create browser windows.
// Some APIs can only be used after this event occurs.
app.on("ready", async () => {
  createWindow();
  const opacity = store.get('opacity') as number;
  mainWindow.webContents.on("devtools-closed", () => {
    mainWindow.setOpacity(opacity);
    mainWindow.setIgnoreMouseEvents(true);
  });

  app.on("activate", function () {
    // On macOS it's common to re-create a window in the app when the
    // dock icon is clicked and there are no other windows open.
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
  });
});

// Quit when all windows are closed, except on macOS. There, it's common
// for applications and their menu bar to stay active until the user quits
// explicitly with Cmd + Q.
app.on("window-all-closed", () => {
  if (process.platform !== "darwin") {
    app.quit();
  }
});
