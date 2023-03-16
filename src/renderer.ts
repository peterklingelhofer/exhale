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
let canvasWidth = 1;
let canvasHeight = 1;
let halfCanvasHeight = 0;
let state = State.POST_EXHALE;
let startFrame = 0;
let endFrame = 0;
let radius = 0;
let color: Color = colorInhale;
const recordedAnimation: ImageData[] = [];
let recording = true;

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

const offscreenCanvas = document.createElement("canvas");
offscreenCanvas.width = canvasWidth;
offscreenCanvas.height = canvasHeight;
const offscreenCtx = offscreenCanvas.getContext("2d");

function drawOffscreen(): void {
  offscreenCtx.fillStyle = BACKDROP_COLOR;
  offscreenCtx.fillRect(0, 0, canvasWidth, canvasHeight);

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

  if (color !== offscreenCtx.fillStyle) {
    offscreenCtx.fillStyle = color;
  }

  if (circleOrRectangle === Shape.CIRCLE) {
    const centerX = canvasWidth / 2;
    const centerY = canvasHeight / 2;
    const startAngle = 0;
    const endAngle = 2 * Math.PI;
    const isCounterClockwise = false;
    offscreenCtx.beginPath();
    offscreenCtx.arc(centerX, centerY, radius, startAngle, endAngle, isCounterClockwise);
    offscreenCtx.fill();
  } else {
    const twiceRadius = radius * 2;
    offscreenCtx.fillRect(0, canvasHeight - twiceRadius, canvasWidth, twiceRadius);
  }

  if (frameCount >= endFrame) {
    startFrame = frameCount;
    state = progressState(state);
    if (state === State.POST_EXHALE && recording) {
      recording = false;
      totalFrames = frameCount + 1;
    }
  }
}

// function playRecordedAnimation(): void {
//   if (frameCount >= totalFrames) {
//     frameCount = 0;
//   }
//   ctx.drawImage(offscreenCanvas, 0, 0, canvasWidth, canvasHeight);
//   frameCount++;
// }

let frameCount = 0;
let totalFrames = 0;
const FRAME_INTERVAL = Math.floor(1000 / FRAMES_PER_SECOND);
function playRecordedAnimation(): void {
  ctx.drawImage(offscreenCanvas, 0, 0, canvasWidth, canvasHeight);
}

function animate(): void {
  setTimeout(() => {
    if (recording) {
      drawOffscreen();
      frameCount++;
      if (frameCount >= totalFrames) {
        recording = false;
      }
    } else {
      playRecordedAnimation();
    }
    requestAnimationFrame(animate);
  }, FRAME_INTERVAL);
}

resizeCanvas();
requestAnimationFrame(animate);
