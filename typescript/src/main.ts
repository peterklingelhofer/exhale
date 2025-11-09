// main.ts
import { app, globalShortcut, BrowserWindow } from "electron";
import * as path from "path";

let mainWindow: BrowserWindow;
let isDevToolsOpen = true;
let opacity: number = 1.0;
let isRecreatingWindow = false;

function createWindow() {
  mainWindow = new BrowserWindow({
    alwaysOnTop: true,
    height: 600,
    movable: false,
    resizable: true,
    show: true,
    titleBarStyle: isDevToolsOpen ? undefined : "hidden",
    webPreferences: {
      preload: path.join(__dirname, "preload.js"),
      nodeIntegration: true,
    },
    width: 800,
    transparent: true,
  });

  mainWindow.loadFile(path.join(__dirname, "../index.html"));

  if (isDevToolsOpen) {
    mainWindow.webContents.openDevTools();
  }

  mainWindow.setAlwaysOnTop(true, "floating", 1);
  mainWindow.setVisibleOnAllWorkspaces(true, { visibleOnFullScreen: true });
  mainWindow.setFullScreenable(false);
  mainWindow.setFocusable(true);
  mainWindow.setIgnoreMouseEvents(!isDevToolsOpen);
  mainWindow.removeMenu();
  mainWindow.setOpacity(isDevToolsOpen ? 1.0 : opacity);

  mainWindow.webContents.on("devtools-opened", () => {
    if (!isDevToolsOpen) {
      isDevToolsOpen = true;
      recreateWindow();
    }
  });

  mainWindow.webContents.on("devtools-closed", () => {
    if (isDevToolsOpen) {
      isDevToolsOpen = false;
      recreateWindow();
    }
  });
}

function recreateWindow() {
  // Save current window state
  const bounds = mainWindow.getBounds();

  isRecreatingWindow = true;

  // Destroy the old window
  mainWindow.destroy();

  // Create new window with updated titleBarStyle
  createWindow();

  // Restore window position and size
  mainWindow.setBounds(bounds);

  isRecreatingWindow = false;
}

// This method will be called when Electron has finished
// initialization and is ready to create browser windows.
// Some APIs can only be used after this event occurs.
app.on("ready", async () => {
  createWindow();

  opacity = Number(await mainWindow.webContents.executeJavaScript(
    'localStorage.getItem("opacity");',
    true
  )) || 1.0;

  globalShortcut.register("CommandOrControl+Shift+I", () => {
    if (isDevToolsOpen) {
      mainWindow.webContents.closeDevTools();
    } else {
      mainWindow.webContents.openDevTools();
    }
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
  if (!isRecreatingWindow && process.platform !== "darwin") {
    app.quit();
  }
});
