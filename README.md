# exhale

![exhale](https://github.com/peterklingelhofer/exhale/assets/60944077/8c2ec6cd-d725-4b26-840e-9bf31f0796b5)

## Introduction

Research indicates we blink less and breathe more shallowly when we are looking at screens. This app is intended as a friendly indicator and reminder to continue to take full and deep breaths.

Each of these implementations allows users to set an inhale, inhale hold, exhale, and exhale hold duration, in seconds, to fit their needs. For beginners who might be curious what a good starting value might be for these, I recommend keeping it simple, using `4` for the in duration and `4` for the out duration. Eventually you can work your way up to `6` and `8`, and set the out duration to be twice that of the in duration to facilitate activation of the parasympathetic nervous system. Some users might like to start out with box breathing, which is inhale `4`, inhale hold `4`, exhale `4`, exhale hold `4`. Remember, if intense feelings arise while practicing, taking a break is encouraged - it's important to not overdo it.

## Disclaimer

The information and guidance provided by this breathing app are intended for general informational purposes only and should not be construed as medical advice, diagnosis, or treatment. The creator of this app is not a medical professional, and the app is not a substitute for professional medical advice or consultation with a qualified healthcare provider. Always seek the advice of a physician or other qualified healthcare provider with any questions you may have regarding a medical condition or health objectives. Do not disregard or delay seeking professional medical advice because of the information or suggestions provided by this app. In the event of a medical emergency, call your doctor or dial your local emergency number immediately. Use of this app is at your own risk, and the creator assumes no responsibility for any adverse effects or consequences resulting from its use.

## Swift App Usage

![circle-swift](https://user-images.githubusercontent.com/60944077/226204981-f390facc-4f6c-4bec-8784-23203aa64efc.gif)
![rectangle-swift](https://user-images.githubusercontent.com/60944077/226204986-7522cb4d-7df1-4d65-96de-e629197e9854.gif)
<img width="738" alt="settings-swift" src="https://github.com/peterklingelhofer/exhale/assets/60944077/431af8a9-7ba9-481b-8d58-71d4bfa0074e">



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

Modify settings by going to **Application** (found in the top right via `>>`) > **Local Storage**. While the Developer Tools are open, you can resize the window, and opacity values are ignored, so you can position the window and change settings to your liking, and then close the Dev Tools window by clicking the `x` in the top right, or use <kbd>F12</kbd> (Linux/Windows) or <kbd>Cmd</kbd> + <kbd>Shift</kbd> + <kbd>I</kbd> (macOS) to toggle Developer Tools to [access and modify these settings](https://developer.chrome.com/docs/devtools/storage/localstorage/#edit), and <kbd>Ctrl</kbd> + <kbd>R</kbd> (Linux/Windows) or <kbd>Cmd</kbd> + <kbd>R</kbd> (macOS) to refresh the app to use your newly selected settings. If no settings appear on the first run of the application, you can manually add them), following the format of the `storedValues` variable in [`/src/renderer.ts`](https://github.com/peterklingelhofer/exhale/blob/main/src/renderer.ts). To add them manually, go to the **Console** and copy paste the following code into the console and press <kbd>Enter</kbd> or <kbd>Return</kbd> to populate your `localStorage` (these are the defaults as of the time of writing):
```ts
localStorage = {
  colorExhale = "rgb(0, 0, 255)",
  colorInhale = "rgb(255, 0, 0)",
  colorStyle = "linear", // can be "linear" or "constant"
  shape = "fullscreen", // can be "circle" or "rectangle" or "fullscreen"
  durationInhale = 5,
  durationIn2Out = 0,
  durationExhale = 10,
  durationOut2In = 0,
  opacity = 0.25,
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

Modify variables at the top of the file for preferred in and out duration, in seconds.

For the full-screen resizable version, use, `IS_FULL_SCREEN = True` which makes the window entirely resizable by clicking and dragging from the corners.
