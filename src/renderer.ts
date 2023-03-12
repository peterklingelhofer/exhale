// This file is required by the index.html file and will
// be executed in the renderer process for that window.
// No Node.js APIs are available in this process unless
// nodeIntegration is set to true in webPreferences.
// Use preload.js to selectively enable features
// needed in the renderer process.
const FRAMES_PER_SECOND = 60;
type Color = string | CanvasGradient | CanvasPattern;
enum State {
  INHALE,
  POST_INHALE,
  EXHALE,
  POST_EXHALE,
}
function map(
  value: number,
  start1: number,
  stop1: number,
  start2: number,
  stop2: number
): number {
  return ((value - start1) / (stop1 - start1)) * (stop2 - start2) + start2;
}

const canvas = document.createElement("canvas");
document.body.appendChild(canvas);
const ctx = canvas.getContext("2d");

const colorExhale: Color = localStorage.colorExhale || "rgb(0, 221, 255)";
const colorInhale: Color = localStorage.colorInhale || "rgb(168, 50, 150)";
const durationExhale = +localStorage.durationExhale || 10;
const durationInhale = +localStorage.durationInhale || 5;
const durationPostExhale = +localStorage.durationPostExhale || 0;
const durationPostInhale = +localStorage.durationPostInhale || 0;
const opacity = +localStorage.opacity || 0.1;

Object.assign(localStorage, {
  durationInhale,
  durationExhale,
  durationPostExhale,
  durationPostInhale,
  colorExhale,
  colorInhale,
  opacity,
});
let canvasWidth = 0;
let canvasHeight = 0;
let halfCanvasHeight = 0;
let state = State.INHALE;
let startFrame = 0;
let endFrame = 0;
let radius = 0;
let color: Color = colorInhale;

const stateAfterInhale: State =
  durationPostInhale > 0 ? State.POST_INHALE : State.EXHALE;
const stateAfterExhale: State =
  durationPostExhale > 0 ? State.POST_EXHALE : State.INHALE;
function progressState(state: State): State {
  switch (state) {
    case State.INHALE:
      return stateAfterInhale;
    case State.POST_INHALE:
      return State.EXHALE;
    case State.EXHALE:
      return stateAfterExhale;
    case State.POST_EXHALE:
      return State.INHALE;
  }
}

function resizeCanvas() {
  canvasWidth = canvas.width = window.innerWidth;
  canvasHeight = canvas.height = window.innerHeight;
  halfCanvasHeight = canvasHeight / 2;
}
window.addEventListener("resize", resizeCanvas);

function draw() {
  let elapsed = 0;
  ctx.fillStyle = "black";
  ctx.fillRect(0, 0, canvasWidth, canvasHeight);

  switch (state) {
    case State.INHALE:
      color = colorInhale;
      endFrame = startFrame + durationInhale * FRAMES_PER_SECOND;
      elapsed = (frameCount - startFrame) / FRAMES_PER_SECOND;
      radius = map(elapsed, 0, durationInhale, 0, halfCanvasHeight);
      radius = Math.min(radius, halfCanvasHeight);
      break;
    case State.POST_INHALE:
      endFrame = startFrame + (durationPostInhale + 0.1) * FRAMES_PER_SECOND;
      radius = halfCanvasHeight;
      break;
    case State.EXHALE:
      color = colorExhale;
      endFrame = startFrame + durationExhale * FRAMES_PER_SECOND;
      elapsed = (frameCount - startFrame) / FRAMES_PER_SECOND;
      radius = map(elapsed, 0, durationExhale, halfCanvasHeight, 0);
      radius = Math.max(radius, 0);
      break;
    case State.POST_EXHALE:
      color = "black";
      endFrame = startFrame + (durationPostExhale + 0.1) * FRAMES_PER_SECOND;
      radius = halfCanvasHeight;
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
