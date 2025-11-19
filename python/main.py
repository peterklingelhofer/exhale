import math
import time
import tkinter as tk
from PyQt5.QtWidgets import QApplication, QWidget
from PyQt5.QtGui import QPainter, QColor, QPen
from PyQt5.QtCore import Qt, QTimer
import sys
import signal

# configure parameters and constants
INHALE_DURATION = 4  # set inhale (up animation) duration in seconds
EXHALE_DURATION = 6  # set exhale (down animation) duration in seconds
POST_INHALE_HOLD = 0  # set hold time after inhale (up animation) duration in seconds
POST_EXHALE_HOLD = 0  # set hold time after exhale (down animation) duration in seconds
SHAPE = "circle"  # set shape: "bars" or "circle"
IS_FULL_SCREEN = False  # toggles full screen mode
SIDE_WIDTH = 20  # set width (only if IS_FULL_SCREEN is False, recommended values between 10 and 20)
COLOR = "#5D3FD3"  # set color, "#F4DFC9" may be congenial for full screen and lower opacity values
OPACITY = .25  # set transparency (between 0 and 1)
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
    # Prevent the window from taking focus
    window.attributes("-type", "dock")
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


class CircleWidget(QWidget):
    def __init__(self):
        super().__init__()
        self.radius = 150
        self.minRadius = 150
        self.maxRadius = min(screenWidth, screenHeight) // 4
        self.radiusRange = self.maxRadius - self.minRadius

        # Setup window
        self.setWindowFlags(Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.WindowDoesNotAcceptFocus | Qt.WindowTransparentForInput | Qt.BypassWindowManagerHint)
        self.setAttribute(Qt.WA_TranslucentBackground)
        self.setGeometry(0, 0, screenWidth, screenHeight)

        # Animation state
        self.phase = 'inhale'  # 'inhale', 'post_inhale', 'exhale', 'post_exhale'
        self.frameIndex = 0

        # Calculate frames
        self.framesUp, self.framesDown = calculateFramesPerPhase()
        self.incrementUp, self.incrementDown = calculateIncrementPerFrame(self.framesUp, self.framesDown)

        # Setup timer
        self.timer = QTimer()
        self.timer.timeout.connect(self.updateAnimation)
        self.frameDelay = int(1000 / FRAME_RATE)  # milliseconds
        self.timer.start(self.frameDelay)

        # Opacity
        if OPACITY < 1:
            self.setWindowOpacity(OPACITY)

    def paintEvent(self, event):
        painter = QPainter(self)
        painter.setRenderHint(QPainter.Antialiasing)

        # Set color
        color = QColor(COLOR)
        painter.setBrush(color)
        painter.setPen(QPen(Qt.NoPen))

        # Draw circle
        centerX = screenWidth // 2
        centerY = screenHeight // 2
        painter.drawEllipse(int(centerX - self.radius), int(centerY - self.radius),
                          int(self.radius * 2), int(self.radius * 2))

    def updateAnimation(self):
        if self.phase == 'inhale':
            if self.frameIndex < self.framesUp:
                progress = math.sin(self.incrementUp * self.frameIndex)
                self.radius = self.minRadius + (progress * self.radiusRange)
                self.frameIndex += 1
            else:
                self.phase = 'post_inhale'
                self.frameIndex = 0
                if POST_INHALE_HOLD > 0:
                    self.timer.stop()
                    QTimer.singleShot(int(POST_INHALE_HOLD * 1000), self.resumeFromPostInhale)
        elif self.phase == 'post_inhale':
            self.phase = 'exhale'
            self.frameIndex = 0
        elif self.phase == 'exhale':
            if self.frameIndex < self.framesDown:
                progress = math.sin(self.incrementDown * self.frameIndex)
                self.radius = self.maxRadius - (progress * self.radiusRange)
                self.frameIndex += 1
            else:
                self.phase = 'post_exhale'
                self.frameIndex = 0
                if POST_EXHALE_HOLD > 0:
                    self.timer.stop()
                    QTimer.singleShot(int(POST_EXHALE_HOLD * 1000), self.resumeFromPostExhale)
        elif self.phase == 'post_exhale':
            self.phase = 'inhale'
            self.frameIndex = 0

        self.update()

    def resumeFromPostInhale(self):
        self.phase = 'exhale'
        self.frameIndex = 0
        self.timer.start(self.frameDelay)

    def resumeFromPostExhale(self):
        self.phase = 'inhale'
        self.frameIndex = 0
        self.timer.start(self.frameDelay)


def animateCircle():
    # Set up signal handler to allow Ctrl+C to work
    signal.signal(signal.SIGINT, signal.SIG_DFL)

    app = QApplication(sys.argv)
    widget = CircleWidget()
    widget.show()

    # Use a timer to allow Python to process signals
    timer = QTimer()
    timer.start(500)
    timer.timeout.connect(lambda: None)

    sys.exit(app.exec_())


def animateBars():
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


def animate():
    if SHAPE == "circle":
        animateCircle()
    else:
        animateBars()


if __name__ == '__main__':
    animate()
