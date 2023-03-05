import math
import time
import tkinter as tk

# configure parameters and constants
COLOR = "#000"  # set color, "#F4DFC9" may be congenial for full screen and lower opacity values
OPACITY = 1  # set transparency (between 0 and 1)
INHALE_DURATION = 4  # set inhale (up animation) duration in seconds
EXHALE_DURATION = 8  # set exhale (down animation) duration in seconds
IS_FULL_SCREEN = False  # toggles full screen mode
SIDE_WIDTH = 20  # set width (only if IS_FULL_SCREEN is False, recommended values between 10 and 20)
FRAME_RATE = 30  # set frame rate

# get screen dimensions
root = tk.Tk()
root.withdraw()
screen_height = root.winfo_screenheight()
screen_width = root.winfo_screenwidth()
animation_width = (SIDE_WIDTH, screen_width)[IS_FULL_SCREEN]  # use SIDE_WIDTH if we're not in full screen mode


# create windows to hold tint overlays
def create_window(geometry):
    window = tk.Toplevel()
    window.geometry(geometry)
    window.attributes("-topmost", True)
    window.overrideredirect(True)
    window.resizable(IS_FULL_SCREEN, IS_FULL_SCREEN)
    if OPACITY < 1:
        window.wm_attributes("-alpha", OPACITY)

    return window


# create canvas on left/right side to draw horizontal line on
def create_canvas(overlay):
    canvas = tk.Canvas(overlay, width=animation_width, height=screen_height, highlightthickness=0)
    canvas.pack()
    return canvas


def create_rectangle(canvas):
    return canvas.create_rectangle(0, screen_height, animation_width, screen_height // 2, fill=COLOR, outline="")


# calculate the number of frames to use for each phase of the animation
def calculate_frames_per_phase():
    return math.ceil(INHALE_DURATION * FRAME_RATE), math.ceil(EXHALE_DURATION * FRAME_RATE)


# calculate the increment of frames to use for each phase of the animation
def calculate_increment_per_frame(up, down):
    half_pie = math.pi / 2
    return half_pie / up, half_pie / down


# create windows/canvases/rectangles for tint overlays
def create_screen(window_parameters):
    window = create_window(window_parameters)
    canvas = create_canvas(window)
    rectangle = create_rectangle(canvas)
    return window, canvas, rectangle


# define animation function to move the horizontal line up and down
def animate():
    frames_up, frames_down = calculate_frames_per_phase()
    increment_up, increment_down = calculate_increment_per_frame(frames_up, frames_down)
    window_left, canvas_left, tinted_rect_left = create_screen(f"{animation_width}x{screen_height}+0+0")
    if not IS_FULL_SCREEN:
        window_right, canvas_right, tinted_rect_right = create_screen(f"{animation_width}x{screen_height}+{screen_width - animation_width}+0")

    # animate line moving up and down on left overlay
    while True:
        for i in range(frames_up):
            sin_value = math.sin(increment_up * i)
            height = screen_height - (sin_value * screen_height)

            canvas_left.coords(tinted_rect_left, 0, height, animation_width, screen_height)
            if not IS_FULL_SCREEN:
                canvas_right.coords(tinted_rect_right, 0, height, animation_width, screen_height)

            window_left.update()
            if not IS_FULL_SCREEN:
                window_right.update()

            time.sleep(INHALE_DURATION / frames_up)
        for i in range(frames_down):
            sin_value = math.sin(increment_down * i)
            height = (sin_value * screen_height)

            canvas_left.coords(tinted_rect_left, 0, height, animation_width, screen_height)
            if not IS_FULL_SCREEN:
                canvas_right.coords(tinted_rect_right, 0, height, animation_width, screen_height)

            window_left.update()
            if not IS_FULL_SCREEN:
                window_right.update()

            time.sleep(EXHALE_DURATION / frames_down)


# press the green button in the gutter to run the script
if __name__ == '__main__':
    # start animation
    animate()
