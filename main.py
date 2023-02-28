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

# create window to hold tint overlay
overlay = tk.Toplevel()
overlay.geometry(f"{screen_width}x{screen_height}+0+0")
overlay.wm_attributes("-alpha", 0.3)  # set transparency to 30%
overlay.attributes("-topmost", True)
overlay.overrideredirect(True)
overlay.resizable(True, True)

# create canvas to draw horizontal line on
canvas = tk.Canvas(overlay, width=screen_width, height=screen_height, highlightthickness=0)
canvas.pack()

# draw initial tinted rectangle on canvas
tinted_rect = canvas.create_rectangle(0, screen_height, screen_width, screen_height // 2, fill=tint_color, outline="")


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

    # animate line moving up and down
    while True:
        for i in range(frames_up):
            sin_value = math.sin(increment_up * i)
            height = screen_height - (sin_value * screen_height)
            canvas.coords(tinted_rect, 0, height, screen_width, screen_height)
            overlay.update()
            time.sleep(duration_up / frames_up)
        for i in range(frames_down):
            sin_value = math.sin(increment_down * i)
            height = (sin_value * screen_height)
            canvas.coords(tinted_rect, 0, height, screen_width, screen_height)
            overlay.update()
            time.sleep(duration_down / frames_down)


# press the green button in the gutter to run the script
if __name__ == '__main__':
    # start animation
    animate()
