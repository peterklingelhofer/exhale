import time
import tkinter as tk

# set tint color
tint_color = "#F4DFC9"

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

    # animate line moving up and down
    while True:
        for i in range(0, screen_height):
            canvas.coords(tinted_rect, 0, screen_height - i, screen_width, screen_height)
            overlay.update()
            time.sleep(duration_up / screen_height)
        for i in range(screen_height - 1, line_height - 1, -1):
            canvas.coords(tinted_rect, 0, screen_height - i, screen_width, screen_height)
            overlay.update()
            time.sleep(duration_down / (screen_height - line_height))
        for i in range(line_height - 1, -1, -1):
            canvas.coords(tinted_rect, 0, screen_height - i, screen_width, screen_height)
            overlay.update()
            time.sleep(duration_up / line_height)



# Press the green button in the gutter to run the script.
if __name__ == '__main__':
    # start animation
    animate()
