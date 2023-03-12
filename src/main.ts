import { app, BrowserWindow } from "electron";
import * as path from "path";

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
      nodeIntegration: true,
      preload: path.join(__dirname, "preload.js"),
    },
    width: 800,
  });

  mainWindow.loadFile(path.join(__dirname, "../index.html"));
  mainWindow.webContents.openDevTools();
  mainWindow.setAlwaysOnTop(true, "floating", 1);
  mainWindow.setVisibleOnAllWorkspaces(true, { visibleOnFullScreen: true });
  mainWindow.setFullScreenable(false);
  mainWindow.setFocusable(true);
  mainWindow.setIgnoreMouseEvents(true);
  mainWindow.removeMenu();
}

// This method will be called when Electron has finished
// initialization and is ready to create browser windows.
// Some APIs can only be used after this event occurs.
app.whenReady().then(() => {
  createWindow();
  mainWindow.webContents
    .executeJavaScript('localStorage.getItem("opacity");', true)
    .then((opacity) => {
      mainWindow.webContents.on("devtools-focused", () => {
        mainWindow.setOpacity(1.0);
        mainWindow.setIgnoreMouseEvents(false);
      });
      mainWindow.webContents.on("devtools-closed", () => {
        mainWindow.setOpacity(Number(opacity));
        mainWindow.setIgnoreMouseEvents(true);
      });
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
