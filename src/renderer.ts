// This file is required by the index.html file and will
// be executed in the renderer process for that window.
// No Node.js APIs are available in this process unless
// nodeIntegration is set to true in webPreferences.
// Use preload.js to selectively enable features
// needed in the renderer process.

type Color = string | CanvasGradient | CanvasPattern;
enum State {
  INHALE,
  POST_INHALE,
  EXHALE,
  POST_EXHALE,
}
function progressState(state: State): State {
  switch (state) {
    case State.INHALE:
      return State.POST_INHALE;
    case State.POST_INHALE:
      return State.EXHALE;
    case State.EXHALE:
      return State.POST_EXHALE;
    case State.POST_EXHALE:
      return State.INHALE;
  }
}
const canvas = document.createElement("canvas");
document.body.appendChild(canvas);
const ctx = canvas.getContext("2d");

const storedValues: {
  colorExhale: Color;
  colorInhale: Color;
  colorPause: Color;
  durationExhale: number;
  durationInhale: number;
  durationPostExhale: number;
  durationPostInhale: number;
  opacity: number;
} = {
  colorExhale: localStorage.colorExhale || "rgba(168,50,150,1)",
  colorInhale: localStorage.colorInhale || "rgba(0,221,255,1)",
  colorPause: localStorage.colorPause || "rgba(0,221,255,1)",
  durationExhale: +localStorage.durationExhale || 10,
  durationInhale: +localStorage.durationInhale || 5,
  durationPostExhale: +localStorage.durationPostExhale || 0,
  durationPostInhale: +localStorage.durationPostInhale || 0,
  opacity: +localStorage.opacity || 0.1,
};

Object.assign(localStorage, storedValues);

let state = State.INHALE;
let startFrame = 0;
let endFrame = 0;
let radius = 0;
let color: Color = "black";

function resizeCanvas() {
  canvas.width = window.innerWidth;
  canvas.height = window.innerHeight;
}

window.addEventListener("resize", resizeCanvas);
resizeCanvas();

function map(
  value: number,
  start1: number,
  stop1: number,
  start2: number,
  stop2: number
) {
  return ((value - start1) / (stop1 - start1)) * (stop2 - start2) + start2;
}

function draw() {
  const {
    durationInhale,
    durationExhale,
    durationPostExhale,
    durationPostInhale,
    colorExhale,
    colorInhale,
    colorPause,
  } = storedValues;
  let elapsed = 0;
  ctx.fillStyle = color;
  ctx.fillRect(0, 0, canvas.width, canvas.height);

  switch (state) {
    case State.INHALE:
      color = colorExhale;
      endFrame = startFrame + durationInhale * 60;
      elapsed = (frameCount - startFrame) / 60;
      radius = map(elapsed, 0, durationInhale, 0, canvas.height / 2);
      radius = Math.min(radius, canvas.height / 2);
      break;
    case State.POST_INHALE:
      color = colorPause;
      endFrame = startFrame + (durationPostInhale + 0.1) * 60;
      radius = canvas.height / 2;
      break;
    case State.EXHALE:
      color = colorInhale;
      endFrame = startFrame + durationExhale * 60;
      elapsed = (frameCount - startFrame) / 60;
      radius = map(elapsed, 0, durationExhale, canvas.height / 2, 0);
      radius = Math.max(radius, 0);
      break;
    case State.POST_EXHALE:
      color = colorPause;
      endFrame = startFrame + (durationPostExhale + 0.1) * 60;
      radius = canvas.height / 2;
      break;
  }

  ctx.fillStyle = "black";
  ctx.fillRect(0, canvas.height - radius * 2, canvas.width, radius * 2);
  if (frameCount >= endFrame) {
    startFrame = frameCount;
    state = progressState(state);
  }
}

let frameCount = 0;
setInterval(() => {
  draw();
  frameCount++;
}, 1000 / 60);
