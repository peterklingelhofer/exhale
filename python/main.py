import math
import time
import tkinter as tk

# configure parameters and constants
INHALE_DURATION = 5  # set inhale (up animation) duration in seconds
EXHALE_DURATION = 10  # set exhale (down animation) duration in seconds
POST_INHALE_HOLD = 0  # set hold time after inhale (up animation) duration in seconds
POST_EXHALE_HOLD = 0  # set hold time after exhale (down animation) duration in seconds
IS_FULL_SCREEN = False  # toggles full screen mode
SIDE_WIDTH = 20  # set width (only if IS_FULL_SCREEN is False, recommended values between 10 and 20)
COLOR = "#000"  # set color, "#F4DFC9" may be congenial for full screen and lower opacity values
OPACITY = 1  # set transparency (between 0 and 1)
FRAME_RATE = 30  # set frame rate

root = tk.Tk()
root.withdraw()
screenHeight = root.winfo_screenheight()
screenWidth = root.winfo_screenwidth()
animationWidth = (SIDE_WIDTH, screenWidth)[IS_FULL_SCREEN]  # use SIDE_WIDTH if we're not in full screen mode


def createWindow(geometry):
    window = tk.Toplevel()
    window.geometry(geometry)
    window.attributes("-topmost", True)
    window.overrideredirect(True)
    window.resizable(IS_FULL_SCREEN, IS_FULL_SCREEN)
    if OPACITY < 1:
        window.wm_attributes("-alpha", OPACITY)
    return window


def createCanvas(overlay):
    canvas = tk.Canvas(overlay, width=animationWidth, height=screenHeight, highlightthickness=0)
    canvas.pack()
    return canvas


def createRectangle(canvas):
    return canvas.create_rectangle(0, screenHeight, animationWidth, screenHeight // 2, fill=COLOR, outline="")


def calculateFramesPerPhase():
    return math.ceil(INHALE_DURATION * FRAME_RATE), math.ceil(EXHALE_DURATION * FRAME_RATE)


def calculateIncrementPerFrame(up, down):
    halfPie = math.pi / 2
    return halfPie / up, halfPie / down


def createScreen(windowParameters):
    window = createWindow(windowParameters)
    canvas = createCanvas(window)
    rectangle = createRectangle(canvas)
    return window, canvas, rectangle


def updateScreens(canvasLeft, canvasRight, rectangleLeft, rectangleRight, windowLeft, windowRight, y):
    canvasLeft.coords(rectangleLeft, 0, y, animationWidth, screenHeight)
    if not IS_FULL_SCREEN:
        canvasRight.coords(rectangleRight, 0, y, animationWidth, screenHeight)

    windowLeft.update()
    if not IS_FULL_SCREEN:
        windowRight.update()


def animate():
    framesUp, framesDown = calculateFramesPerPhase()
    incrementUp, incrementDown = calculateIncrementPerFrame(framesUp, framesDown)
    windowLeft, canvasLeft, rectangleLeft = createScreen(f"{animationWidth}x{screenHeight}+0+0")
    windowRight, canvasRight, rectangleRight = (
        createScreen(f"{animationWidth}x{screenHeight}+{screenWidth - animationWidth}+0")
        if not IS_FULL_SCREEN
        else (None, None, None))

    while True:
        for i in range(framesUp):
            y = screenHeight - (math.sin(incrementUp * i) * screenHeight)
            updateScreens(canvasLeft, canvasRight, rectangleLeft, rectangleRight, windowLeft, windowRight, y)
            time.sleep(INHALE_DURATION / framesUp)

        time.sleep(POST_INHALE_HOLD)

        for i in range(framesDown):
            y = math.sin(incrementDown * i) * screenHeight
            updateScreens(canvasLeft, canvasRight, rectangleLeft, rectangleRight, windowLeft, windowRight, y)
            time.sleep(EXHALE_DURATION / framesDown)

        time.sleep(POST_EXHALE_HOLD)


if __name__ == '__main__':
    animate()
