# exhale

## Introduction

Research indicates we blink less and breathe more shallowly when we are looking at screens. This app is intended as a friendly indicator and reminder to continue to take full and deep breaths.

[<img src="https://user-images.githubusercontent.com/60944077/232312847-df673556-fb5e-49b4-8037-4d38267e6e18.png"  width="157" height="63"></img>](https://apps.apple.com/us/app/exhale-breath/id6447758995?mt=12)

## Swift App Usage

![circle-swift](https://user-images.githubusercontent.com/60944077/226204981-f390facc-4f6c-4bec-8784-23203aa64efc.gif)
![rectangle-swift](https://user-images.githubusercontent.com/60944077/226204986-7522cb4d-7df1-4d65-96de-e629197e9854.gif)
<img width="600" alt="Screen Shot 2023-03-22 at 8 54 17 PM" src="https://user-images.githubusercontent.com/60944077/227079185-fb5d5fc3-e966-4488-a68b-8dc799651a02.png">


Note: This is the macOS implementation.

```sh
git clone https://github.com/peterklingelhofer/exhale.git
cd exhale
cd swift
xed .
```

## TypeScript / Electron Usage

![exhaleElectron](https://user-images.githubusercontent.com/60944077/224524962-56da25cc-e3d9-4d4b-9171-f185be9d709c.gif)
![exhaleElectronCircular](https://user-images.githubusercontent.com/60944077/224865780-0e61721e-2345-49aa-830d-0e157b6f4366.gif)

Note: This is the Linux and Windows implementation. macOS will build but it is not very performant and is far more CPU-intensive than the native Swift build.

```sh
git clone https://github.com/peterklingelhofer/exhale.git
cd exhale
cd typescript
yarn
yarn start
```

To recompile automatically and use [electron-reload](https://github.com/yan-foto/electron-reload), run in a separate terminal:

```sh
yarn run watch
```

Modify settings by going to **Application** (found in the top right via `>>`) > **Local Storage**. Use <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>I</kbd> or <kbd>Cmd</kbd> + <kbd>Shift</kbd> + <kbd>I</kbd>to open and close these Developer Tools to [access and modify these settings](https://developer.chrome.com/docs/devtools/storage/localstorage/#edit), and <kbd>Ctrl</kbd> + <kbd>R</kbd> or <kbd>Cmd</kbd> + <kbd>R</kbd> to refresh the app to use your newly selected settings. If no settings appear on the first run of the application, you can manually add them), following the format of the `storedValues` variable in [`/src/renderer.ts`](https://github.com/peterklingelhofer/exhale/blob/main/src/renderer.ts). To add them manually, go to the **Console** and copy paste the following code into the console and press <kbd>Enter</kbd> or <kbd>Return</kbd> to populate your `localStorage` (these are the defaults as of the time of writing):
```ts
localStorage = {
  colorExhale = "rgb(0, 221, 255)",
  colorInhale = "rgb(168, 50, 150)",
  colorStyle = "linear", // can be "linear" or "constant"
  circleOrRectangle = "rectangle", // can be "circle" or "rectangle"
  durationExhale = 10,
  durationInhale = 5,
  durationPostExhale = 0,
  durationPostInhale = 0,
  opacity = 0.1,
}
```

Once added, you can modify all values from the **Local Storage** pane. Or, if you prefer the terminal, in the **Console** you can write `localStorage.opacity = "0.15"` for example.

<img width="371" alt="Screen Shot 2023-03-11 at 2 12 30 PM" src="https://user-images.githubusercontent.com/60944077/224511531-c0d615a1-1859-47b6-a78b-7d38276d80be.png">

## Python Script Usage

![exhalePython](https://user-images.githubusercontent.com/60944077/222979803-c88ebc65-b799-4ca7-b265-54beb27fcb00.gif)

Note: This implementation seems to work well on Windows and macOS, but not Linux for some reason.

```sh
git clone https://github.com/peterklingelhofer/exhale.git
cd exhale
cd python
python main.py
```

Modify variables at the top of the file for preferred in and out duration, in seconds. For beginners who don't know what a good values for these, I recommend keeping it simple, using `4` for the in duration and `4` for the out duration. Eventually you can work your way up to `6` and `8`, and set the out duration to be twice that of the in duration to facilitate activation of the parasympathetic nervous system.

For the full-screen resizable version, use: `IS_FULL_SCREEN = True`
