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
enum Shape {
  CIRCLE = "circle",
  RECTANGLE = "rectangle",
}
const FRAMES_PER_SECOND = 60;
const BACKDROP_COLOR: Color = "#000";
const calculateEndFrame = (duration: number) =>
  startFrame + duration * FRAMES_PER_SECOND;
const calculateElapsed = (frames: number) =>
  (frames - startFrame) / FRAMES_PER_SECOND;
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

const {
  colorExhale = "rgb(0, 221, 255)",
  colorInhale = "rgb(168, 50, 150)",
  circleOrRectangle = Shape.CIRCLE,
  durationExhale = 10,
  durationInhale = 5,
  durationPostExhale = 0,
  durationPostInhale = 0,
  opacity = 0.1,
} = localStorage;

Object.assign(localStorage, {
  circleOrRectangle,
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
let state = State.POST_EXHALE;
let startFrame = 0;
let endFrame = 0;
let radius = 0;
let color: Color = colorInhale;

function resizeCanvas(): void {
  canvasWidth = canvas.width = window.innerWidth;
  canvasHeight = canvas.height = window.innerHeight;
  halfCanvasHeight = canvasHeight / 2;
}
window.addEventListener("resize", resizeCanvas);

function progressState(state: State): State {
  color =
    state === State.POST_INHALE || state === State.EXHALE
      ? colorExhale
      : colorInhale;

  switch (state) {
    case State.INHALE:
      endFrame =
        durationPostInhale > 0
          ? calculateEndFrame(durationPostInhale)
          : calculateEndFrame(durationExhale);
      return durationPostInhale > 0 ? State.POST_INHALE : State.EXHALE;
    case State.POST_INHALE:
      endFrame = calculateEndFrame(durationExhale);
      return State.EXHALE;
    case State.EXHALE:
      endFrame =
        durationPostExhale > 0
          ? calculateEndFrame(durationPostExhale)
          : calculateEndFrame(durationInhale);
      return durationPostExhale > 0 ? State.POST_EXHALE : State.INHALE;
    case State.POST_EXHALE:
      endFrame = calculateEndFrame(durationInhale);
      return State.INHALE;
  }
}

function draw(): void {
  ctx.fillStyle = BACKDROP_COLOR;
  ctx.fillRect(0, 0, canvasWidth, canvasHeight);

  switch (state) {
    case State.INHALE:
      radius = Math.min(
        map(
          calculateElapsed(frameCount),
          0,
          durationInhale,
          0,
          halfCanvasHeight
        ),
        halfCanvasHeight
      );
      break;
    case State.EXHALE:
      radius = Math.max(
        map(
          calculateElapsed(frameCount),
          0,
          durationExhale,
          halfCanvasHeight,
          0
        ),
        0
      );
      break;
    default:
      break;
  }

  if (color !== ctx.fillStyle) {
    ctx.fillStyle = color;
  }

  if (circleOrRectangle === Shape.CIRCLE) {
    const centerX = canvasWidth / 2;
    const centerY = canvasHeight / 2;
    const startAngle = 0;
    const endAngle = 2 * Math.PI;
    const isCounterClockwise = false;
    ctx.beginPath();
    ctx.arc(centerX, centerY, radius, startAngle, endAngle, isCounterClockwise);
    ctx.fill();
  } else {
    const twiceRadius = radius * 2;
    ctx.fillRect(0, canvasHeight - twiceRadius, canvasWidth, twiceRadius);
  }

  if (frameCount >= endFrame) {
    startFrame = frameCount;
    state = progressState(state);
  }
}

let frameCount = 0;
function animate(): void {
  draw();
  frameCount++;
  requestAnimationFrame(animate);
}

requestAnimationFrame(animate);
