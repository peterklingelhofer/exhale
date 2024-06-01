// main.ts
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
      preload: path.join(__dirname, "preload.js"),
      nodeIntegration: true,
    },
    width: 800,
  });

  mainWindow.loadFile(path.join(__dirname, "../index.html"));
  mainWindow.webContents.openDevTools();

  // Add a delay to ensure DevTools is fully opened before sending the command
  mainWindow.webContents.on("devtools-opened", () => {
    // Focus on the Application panel by simulating the necessary key press
    mainWindow.webContents.devToolsWebContents?.sendInputEvent({
      type: "keyDown",
      keyCode: "p",
      modifiers: ["control", "shift"],
    });
    setTimeout(() => {
      mainWindow.webContents.devToolsWebContents?.sendInputEvent({
        type: "char",
        keyCode: "Application",
      });
      setTimeout(() => {
        mainWindow.webContents.devToolsWebContents?.executeJavaScript(
          `
          (function() {
            const sectionName = 'local-storage';
            const url = 'file://';

            const panel = UI.panels.application;
            const treeOutline = panel.sidebarTree.element.querySelector('.navigator-tree-outline');
            if (!treeOutline) return;

            // Expand Local Storage section
            const localStorageSection = Array.from(treeOutline.children).find(child => child.title === sectionName);
            if (localStorageSection) {
              localStorageSection.expand();

              // Find and select the file URL item
              const fileItem = Array.from(localStorageSection.children).find(child => child.title.includes(url));
              if (fileItem) {
                fileItem.select();
              }
            }
          })();
        `,
          true
        );
      }, 1000); // Adjust delay as necessary for the script to execute after opening Application panel
    }, 500); // Adjust delay as necessary for the key press to be registered
  });

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
  const opacity: string = await mainWindow.webContents.executeJavaScript(
    'localStorage.getItem("opacity");',
    true
  );
  mainWindow.webContents.on("devtools-closed", () => {
    mainWindow.setOpacity(Number(opacity));
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
