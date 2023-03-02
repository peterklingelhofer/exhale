import math
import time
import tkinter as tk

# set tint color
tint_color = "#000"
use_frame_rate = 30

# get screen dimensions
root = tk.Tk()
root.withdraw()
screen_width = root.winfo_screenwidth()
screen_height = root.winfo_screenheight()

# create windows to hold tint overlays
overlay_left = tk.Toplevel()
overlay_right = tk.Toplevel()

# set geometry of left overlay
overlay_left.geometry(f"10x{screen_height}+0+0")
overlay_left.wm_attributes("-alpha", 0.3)  # set transparency to 30%
overlay_left.attributes("-topmost", True)
overlay_left.overrideredirect(True)
overlay_left.resizable(False, False)

# set geometry of right overlay
overlay_right = tk.Toplevel()
overlay_right.geometry(f"10x{screen_height}+{screen_width-10}+0")
overlay_right.wm_attributes("-alpha", 0.3)  # set transparency to 30%
overlay_right.attributes("-topmost", True)
overlay_right.overrideredirect(True)
overlay_right.resizable(False, False)

# create canvas on left overlay to draw horizontal line on
canvas_left = tk.Canvas(overlay_left, width=10, height=screen_height, highlightthickness=0)
canvas_left.pack()

# create canvas on right overlay to draw horizontal line on
canvas_right = tk.Canvas(overlay_right, width=10, height=screen_height, highlightthickness=0)
canvas_right.pack()

# draw initial tinted rectangle on left canvas
tinted_rect_left = canvas_left.create_rectangle(0, screen_height, 10, screen_height // 2, fill=tint_color, outline="")

# draw initial tinted rectangle on right canvas
tinted_rect_right = canvas_right.create_rectangle(0, screen_height, 10, screen_height // 2, fill=tint_color, outline="")


# define animation function to move the horizontal line up and down
def animate():
    duration_up = int(input("Enter the duration (in seconds) for the line to move up: "))
    duration_down = int(input("Enter the duration (in seconds) for the line to move down: "))

    # calculate height of line
    line_height = screen_height // 2

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
