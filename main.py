import math
import time
import tkinter as tk

# set tint color
tint_color = "#F4DFC9"
use_frame_rate = 30

# get screen dimensions
root = tk.Tk()
root.withdraw()
screen_width = root.winfo_screenwidth()
screen_height = root.winfo_screenheight()


# create windows to hold tint overlays
def create_overlay(geometry):
    overlay = tk.Toplevel()
    overlay.geometry(geometry)
    overlay.wm_attributes("-alpha", 0.3)  # set transparency to 30%
    overlay.attributes("-topmost", True)
    overlay.overrideredirect(True)
    overlay.resizable(False, False)

    return overlay


# create canvas on left/right side to draw horizontal line on
def create_canvas(overlay):
    canvas = tk.Canvas(overlay, width=10, height=screen_height, highlightthickness=0)
    canvas.pack()
    return canvas


def create_rectangle(canvas):
    return canvas.create_rectangle(0, screen_height, 10, screen_height // 2, fill=tint_color, outline="")


# create windows/canvases/rectangles for tint overlays
overlay_left = create_overlay(f"10x{screen_height}+0+0")
overlay_right = create_overlay(f"10x{screen_height}+{screen_width-10}+0")
canvas_left = create_canvas(overlay_left)
canvas_right = create_canvas(overlay_right)
tinted_rect_left = create_rectangle(canvas_left)
tinted_rect_right = create_rectangle(canvas_right)


# define animation function to move the horizontal line up and down
def animate():
    duration_up = int(input("Enter the duration (in seconds) for the line to move up: "))
    duration_down = int(input("Enter the duration (in seconds) for the line to move down: "))

    # calculate the number of frames to use for each phase of the animation
    frames_up = math.ceil(duration_up * use_frame_rate)
    frames_down = math.ceil(duration_down * use_frame_rate)

    # calculate the increment for each frame
    increment_up = (math.pi / 2) / frames_up
    increment_down = (math.pi / 2) / frames_down

    # animate line moving up and down on left overlay
    while True:
        for i in range(frames_up):
            sin_value = math.sin(increment_up * i)
            height = screen_height - (sin_value * screen_height)

            canvas_left.coords(tinted_rect_left, 0, height, 10, screen_height)
            canvas_right.coords(tinted_rect_right, 0, height, 10, screen_height)

            overlay_left.update()
            overlay_right.update()

            time.sleep(duration_up / frames_up)
        for i in range(frames_down):
            sin_value = math.sin(increment_down * i)
            height = (sin_value * screen_height)

            canvas_left.coords(tinted_rect_left, 0, height, 10, screen_height)
            canvas_right.coords(tinted_rect_right, 0, height, 10, screen_height)

            overlay_left.update()
            overlay_right.update()

            time.sleep(duration_down / frames_down)


# press the green button in the gutter to run the script
if __name__ == '__main__':
    # start animation
    animate()
