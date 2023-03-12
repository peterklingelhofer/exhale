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
const {
  durationInhale,
  durationExhale,
  durationPostExhale,
  durationPostInhale,
  colorExhale,
  colorInhale,
  colorPause,
} = storedValues;

let state = State.INHALE;
let startFrame = 0;
let endFrame = 0;
let radius = 0;
let color: Color = "black";

let canvasWidth = window.innerWidth;
let canvasHeight = window.innerHeight;

function resizeCanvas() {
  canvasWidth = canvas.width = window.innerWidth;
  canvasHeight = canvas.height = window.innerHeight;
}
window.addEventListener("resize", resizeCanvas);

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
  let elapsed = 0;
  ctx.fillStyle = "black";
  ctx.fillRect(0, 0, canvasWidth, canvasHeight);

  switch (state) {
    case State.INHALE:
      color = colorExhale;
      endFrame = startFrame + durationInhale * 60;
      elapsed = (frameCount - startFrame) / 60;
      radius = map(elapsed, 0, durationInhale, 0, canvasHeight / 2);
      radius = Math.min(radius, canvasHeight / 2);
      break;
    case State.POST_INHALE:
      color = colorPause;
      endFrame = startFrame + (durationPostInhale + 0.1) * 60;
      radius = canvasHeight / 2;
      break;
    case State.EXHALE:
      color = colorInhale;
      endFrame = startFrame + durationExhale * 60;
      elapsed = (frameCount - startFrame) / 60;
      radius = map(elapsed, 0, durationExhale, canvasHeight / 2, 0);
      radius = Math.max(radius, 0);
      break;
    case State.POST_EXHALE:
      color = colorPause;
      endFrame = startFrame + (durationPostExhale + 0.1) * 60;
      radius = canvasHeight / 2;
      break;
  }

  ctx.fillStyle = color;
  ctx.fillRect(0, canvasHeight - radius * 2, canvasWidth, radius * 2);
  if (frameCount >= endFrame) {
    startFrame = frameCount;
    state = progressState(state);
  }
}

let frameCount = 0;
function animate() {
  draw();
  frameCount++;
  requestAnimationFrame(animate);
}

requestAnimationFrame(animate);
