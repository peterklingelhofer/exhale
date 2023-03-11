# exhale

## Introduction

Research indicates we blink less and breathe more shallowly when we are looking at screens. This app is intended as a friendly indicator and reminder to continue to take full and deep breaths.

## Electron Usage

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

Modify settings by going to **Application** (found in the top right via `>>`) > **Local Storage**. Use <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>I</kbd> or <kbd>Cmd</kbd> + <kbd>Shift</kbd> + <kbd>I</kbd>to open and close these settings, and <kbd>Ctrl</kbd> + <kbd>R</kbd> or <kbd>Cmd</kbd> + <kbd>R</kbd> to refresh the app to use your newly selected settings.


## Python Script Usage

```sh
git clone https://github.com/peterklingelhofer/exhale.git
cd exhale
python main.py
```

Modify variables at the top of the file for preferred in and out duration, in seconds. For beginners who don't know what a good values for these, I recommend keeping it simple, using `4` for the in duration and `4` for the out duration. Eventually you can work your way up to `6` and `8`, and set the out duration to be twice that of the in duration to facilitate activation of the parasympathetic nervous system.

For the full-screen resizable version, use: `IS_FULL_SCREEN = True`

![exhaleDemo](https://user-images.githubusercontent.com/60944077/222979803-c88ebc65-b799-4ca7-b265-54beb27fcb00.gif)
