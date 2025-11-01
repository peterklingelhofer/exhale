# exhale

## Introduction

Research indicates we blink less and breathe more shallowly when we are looking at screens. This minimal app is an internal tool I created for myself that I released for the public in case others find it useful, and is intended as a friendly indicator and reminder to continue to take full and deep breaths. As looking at screens for long periods of time is typically less than ideal, this tool is intended as a means to potentially ameliorate those negative impacts.

Each of these implementations allows users to set an inhale, inhale hold, exhale, and exhale hold duration, in seconds, to fit their needs. For beginners who might be curious what a good starting value might be for these, I recommend keeping it simple, using `4` for the in duration and `4` for the out duration. Eventually you can work your way up to `6` and `8`, and set the out duration to be twice that of the in duration to facilitate activation of the parasympathetic nervous system. Some users might like to start out with box breathing, which is inhale `4`, inhale hold `4`, exhale `4`, exhale hold `4`. Remember, if intense feelings arise while practicing, taking a break is encouraged - it's important to not overdo it.

## Disclaimer

The information and guidance provided by this breathing app are intended for general informational purposes only and should not be construed as medical advice, diagnosis, or treatment. The creator of this app is not a medical professional, and the app is not a substitute for professional medical advice or consultation with a qualified healthcare provider. Always seek the advice of a physician or other qualified healthcare provider with any questions you may have regarding a medical condition or health objectives. Do not disregard or delay seeking professional medical advice because of the information or suggestions provided by this app. In the event of a medical emergency, call your doctor or dial your local emergency number immediately. Use of this app is at your own risk, and the creator assumes no responsibility for any adverse effects or consequences resulting from its use.

## Download

[<img src="https://user-images.githubusercontent.com/60944077/232312847-df673556-fb5e-49b4-8037-4d38267e6e18.png"  width="157" height="63"></img>](https://apps.apple.com/us/app/exhale-breath/id6447758995?mt=12)

You can download the build for your respective operating system on the [Releases](https://github.com/peterklingelhofer/exhale/releases) page. Using the latest release is recommended, but if you run into issues you could try a previous release to see if that yields better results. If you do encounter a problem, please [document the issue you encountered](https://github.com/peterklingelhofer/exhale/issues/new).

## Mac App Usage

![circle-swift](https://user-images.githubusercontent.com/60944077/226204981-f390facc-4f6c-4bec-8784-23203aa64efc.gif)
![rectangle-swift](https://user-images.githubusercontent.com/60944077/226204986-7522cb4d-7df1-4d65-96de-e629197e9854.gif)
<img width="677" alt="Settings macOS app" src="https://github.com/user-attachments/assets/76920109-a53b-4959-b556-359ddc14ffe3" />


Note: This is built natively in Swift.

To launch the app on Catalina or newer for the first time, you may have to right click and select "Open" instead of double clicking on it, and you may need to do this twice. That's Apple's take on "security" for non-notarized binaries, or if you are not connected to the Internet.

You can use <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>,</kbd> to toggle settings open and closed. The **Tint** feature (or Pause) can be used to tint your screen the color of your selected background color, or make your screen darker than otherwise possible for nighttime work (which can compound with both [Night Shift](https://support.apple.com/en-us/102191) and [f.lux](https://justgetflux.com/).

#### Global Keyboard Shortcuts:

<kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>A</kbd>: **Start Animation**

<kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>S</kbd>: **Stop Animation**

<kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>D</kbd>: **Tint Screen**

<kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>F</kbd>: **Reset to Defaults**

<kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>W</kbd> or

<kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>,</kbd>: **Open/Close Preferences**


**Notice:** A high opacity value can obscure the Preferences pane in the current workspace.
To change this value back, you can use <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>F</kbd> to **Reset to Defaults**, or if you'd like to persist your other settings:
1. Swipe left or right with four fingers on your trackpad to switch to a different workspace, or four finger swipe up and select an alternate workspace at the top.
2. From the top bar menu, click Preferences to close the Preferences pane in the previous workspace.
3. Access the top bar menu again, click Preferences to open the Preferences pane in the current workspace, and adjust your Opacity value accordingly.
4. Switch back to the original workspace.

                                        
#### Development

```sh
git clone https://github.com/peterklingelhofer/exhale.git
cd exhale
cd swift
xed .
```

## Windows & Linux App Usage

![exhaleElectron](https://user-images.githubusercontent.com/60944077/224524962-56da25cc-e3d9-4d4b-9171-f185be9d709c.gif)
![exhaleElectronCircular](https://user-images.githubusercontent.com/60944077/224865780-0e61721e-2345-49aa-830d-0e157b6f4366.gif)
<img width="912" alt="Screenshot 2024-06-01 at 1 35 36 PM" src="https://github.com/peterklingelhofer/exhale/assets/60944077/b2eb9450-8dcf-4934-b6c9-08328ef6a167">

Note: This implementation is built with TypeScript & Electron. The macOS will build but it is not very performant and is far more CPU-intensive than the native Swift build, and as a result the Swift build is recommended for macOS users.

Modify settings by going to **Application** (found in the top right via `>>`) > **Local storage** > **file://**. While the Developer Tools are open, you can resize the window, and opacity values are ignored, so you can position the window and change settings to your liking, and then close the Dev Tools window by clicking the `x` in the top right, or use <kbd>F12</kbd> or <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>I</kbd>(Linux/Windows) or <kbd>Cmd</kbd> + <kbd>Shift</kbd> + <kbd>I</kbd> (macOS) to toggle Developer Tools to [access and modify these settings](https://developer.chrome.com/docs/devtools/storage/localstorage/#edit), and <kbd>Ctrl</kbd> + <kbd>R</kbd> (Linux/Windows) or <kbd>Cmd</kbd> + <kbd>R</kbd> (macOS) to refresh the app to use your newly selected settings. If no settings appear on the first run of the application, you can manually add them), following the format of the `storedValues` variable in [`/src/renderer.ts`](https://github.com/peterklingelhofer/exhale/blob/main/src/renderer.ts). To add them manually, go to the **Console** and copy paste the following code into the console and press <kbd>Enter</kbd> or <kbd>Return</kbd> to populate your `localStorage` (these are the defaults as of the time of writing):

```ts
localStorage = {
  colorExhale = "rgb(0, 0, 255)",
  colorInhale = "rgb(255, 0, 0)",
  colorStyle = "linear", // can be "linear" or "constant"
  shape = "fullscreen", // can be "circle" or "rectangle" or "fullscreen"
  durationInhale = 5,
  durationPostInhalePause = 0,
  durationExhale = 10,
  durationPostExhalePause = 0,
  opacity = 0.25,
};
```

Once added, you can modify all values from the **Local Storage** pane. Or, if you prefer the terminal, in the **Console** you can write `localStorage.opacity = "0.15"` for example.

<img width="371" alt="Screen Shot 2023-03-11 at 2 12 30 PM" src="https://user-images.githubusercontent.com/60944077/224511531-c0d615a1-1859-47b6-a78b-7d38276d80be.png">

#### Development

```sh
git clone https://github.com/peterklingelhofer/exhale.git
cd exhale
cd typescript
pnpm install
pnpm start
```

To recompile automatically and use [electron-reload](https://github.com/yan-foto/electron-reload), run in a separate terminal:

```sh
pnpm watch
```

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


## Perl Script

Companion repository made in Perl can be found [here](https://github.com/franco3445/Breathing).


## Sponsorship

This app is free and will always be free, but if you want to support me or want a more rapid response time on feature requests, you may buy me a coffee:

<a href="https://www.buymeacoffee.com/peterklingelhofer" target="_blank"><img src="https://cdn.buymeacoffee.com/buttons/v2/default-yellow.png" alt="Buy Me A Coffee" style="height: 60px !important;width: 217px !important;" ></a>
