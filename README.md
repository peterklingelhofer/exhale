# exhale

## Introduction

Research indicates we blink less and breathe more shallowly when we are looking at screens. This app is intended as a friendly indicator and reminder to continue to take full and deep breaths.

## Electron Usage

![exhaleElectron](https://user-images.githubusercontent.com/60944077/224524962-56da25cc-e3d9-4d4b-9171-f185be9d709c.gif)

```sh
git clone https://github.com/peterklingelhofer/exhale.git
cd exhale
yarn
yarn start
```

To recompile automatically and use [electron-reload](https://github.com/yan-foto/electron-reload), run in a separate terminal:

```sh
yarn run watch
```

Modify settings by going to **Application** (found in the top right via `>>`) > **Local Storage**. Use <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>I</kbd> or <kbd>Cmd</kbd> + <kbd>Shift</kbd> + <kbd>I</kbd>to open and close these Developer Tools to [access and modify these settings](https://developer.chrome.com/docs/devtools/storage/localstorage/#edit), and <kbd>Ctrl</kbd> + <kbd>R</kbd> or <kbd>Cmd</kbd> + <kbd>R</kbd> to refresh the app to use your newly selected settings. If no settings appear on the first run of the application, you can manually add them), following the format of the `storedValues` variable in [`/src/renderer.ts`](https://github.com/peterklingelhofer/exhale/blob/main/src/renderer.ts). To add them manually, go to the **Console** and copy paste the following code into the console and press <kbd>Enter</kbd> or <kbd>Return</kbd> to populate your `localStorage` (these are the defaults as of the time of writing):
```js
localStorage = {
  colorExhale: "rgba(168,50,150,1)",
  colorInhale: "rgba(0,221,255,1)",
  colorPause: "rgba(0,221,255,1)",
  durationExhale: "10",
  durationInhale: "5",
  durationPostExhale: "0",
  durationPostInhale: "0",
  opacity: "0.1",
}
```

Once added, you can modify all values from the **Local Storage** pane. Or, if you prefer the terminal, in the **Console** you can write `localStorage.opacity = "0.15"` for example.

<img width="371" alt="Screen Shot 2023-03-11 at 2 12 30 PM" src="https://user-images.githubusercontent.com/60944077/224511531-c0d615a1-1859-47b6-a78b-7d38276d80be.png">


## Python Script Usage

![exhalePython](https://user-images.githubusercontent.com/60944077/222979803-c88ebc65-b799-4ca7-b265-54beb27fcb00.gif)

```sh
git clone https://github.com/peterklingelhofer/exhale.git
cd exhale
python main.py
```

Modify variables at the top of the file for preferred in and out duration, in seconds. For beginners who don't know what a good values for these, I recommend keeping it simple, using `4` for the in duration and `4` for the out duration. Eventually you can work your way up to `6` and `8`, and set the out duration to be twice that of the in duration to facilitate activation of the parasympathetic nervous system.

For the full-screen resizable version, use: `IS_FULL_SCREEN = True`
