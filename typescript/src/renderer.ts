// This file is required by the index.html file and will
// be executed in the renderer process for that window.
// No Node.js APIs are available in this process unless
// nodeIntegration is set to true in webPreferences.
// Use preload.js to selectively enable features
// needed in the renderer process.
// renderer.ts
type Color = string;
enum ColorStyle {
  CONSTANT = "constant",
  LINEAR = "linear",
}
enum Shape {
  CIRCLE = "circle",
  FULLSCREEN = "fullscreen",
  RECTANGLE = "rectangle",
}
const FRAMES_PER_SECOND = 60;
const BACKDROP_COLOR: Color = "#000";

const canvas = document.createElement("canvas");
document.body.appendChild(canvas);
const ctx = canvas.getContext("2d");

console.warn('Adjust these parameters to your liking (e.g. localStorage.opacity = "0.3"):', localStorage);
const {
  colorExhale = "rgb(0, 0, 255)",
  colorInhale = "rgb(255, 0, 0)",
  colorStyle = ColorStyle.LINEAR,
  shape = Shape.FULLSCREEN,
  durationInhale = 5,
  durationIn2Out = 0,
  durationExhale = 10,
  durationOut2In = 0,
  opacity = 0.25,
} = localStorage;

Object.assign(localStorage, {
  shape,
  colorExhale,
  colorInhale,
  colorStyle,
  durationExhale,
  durationIn2Out,
  durationInhale,
  durationOut2In,
  opacity,
});
let canvasWidth = 0;
let canvasHeight = 0;
let halfCanvasHeight = 0;
let radius = 0;
const color: Color = colorInhale;

function resizeCanvas(): void {
  canvasWidth = canvas.width = window.innerWidth;
  canvasHeight = canvas.height = window.innerHeight;
  halfCanvasHeight = canvasHeight / 2;
}
window.addEventListener("resize", resizeCanvas);

function linspace(start: number, stop: number, num: number, endpoint = true) {
  const div = endpoint ? num - 1 : num;
  const step = (stop - start) / div;
  return Array.from(
    {
      length: num,
    },
    (_, i) => start + step * i
  );
}
const timeInn = linspace(
  (7 * Math.PI) / 4,
  (9 * Math.PI) / 4,
  Math.ceil(durationInhale * FRAMES_PER_SECOND) + 1
);
const timeI2O = linspace(
  (1 * Math.PI) / 4,
  (3 * Math.PI) / 4,
  Math.ceil(durationIn2Out * FRAMES_PER_SECOND) + 1
);
const timeOut = linspace(
  (3 * Math.PI) / 4,
  (5 * Math.PI) / 4,
  Math.ceil(durationExhale * FRAMES_PER_SECOND) + 1
);
const timeO2I = linspace(
  (5 * Math.PI) / 4,
  (7 * Math.PI) / 4,
  Math.ceil(durationOut2In * FRAMES_PER_SECOND) + 1
);

timeInn.pop();
timeI2O.pop();
timeOut.pop();
timeO2I.pop();

const indices: Array<number> = [];

// array math is not defined in base javascript >.<
// i wonder if this is slow...
for (let i = 0; i < timeInn.length; i++) {
  indices.push((Math.sin(timeInn[i]) + 1) / 2);
}
for (let i = 0; i < timeI2O.length; i++) {
  indices.push((Math.sin(timeI2O[i]) + 1) / 2);
}
for (let i = 0; i < timeOut.length; i++) {
  indices.push((Math.sin(timeOut[i]) + 1) / 2);
}
for (let i = 0; i < timeO2I.length; i++) {
  indices.push((Math.sin(timeO2I[i]) + 1) / 2);
}
const totalFrames = indices.length;

let totalFrameInd: number;
let transitionValue: number;

function draw(): void {
  ctx.fillStyle = BACKDROP_COLOR;
  ctx.fillRect(0, 0, canvasWidth, canvasHeight);

  let gradient;

  // calculate radius

  // convert the frameCount (special variable) to its position in our totalFrames
  totalFrameInd = frameCount % totalFrames;

  // first determine what "frame" we are on within the animation (analog)
  transitionValue = indices[totalFrameInd];

  // radius is a function of transitionValue
  radius = transitionValue * halfCanvasHeight;

  if (shape === Shape.FULLSCREEN) {
    const inhaleColorComponents = colorInhale.match(/\d+/g).map(Number);
    const exhaleColorComponents = colorExhale.match(/\d+/g).map(Number);
    const interpolatedColor = inhaleColorComponents.map((comp: number, index: string | number) => {
      return comp + (exhaleColorComponents[index] - comp) * transitionValue;
    });

    ctx.fillStyle = `rgb(${interpolatedColor[0]}, ${interpolatedColor[1]}, ${interpolatedColor[2]})`;
    ctx.fillRect(0, 0, canvasWidth, canvasHeight);
  } else if (shape === Shape.CIRCLE) {
    const centerX = canvasWidth / 2;
    const centerY = canvasHeight / 2;
    const startAngle = 0;
    const endAngle = 2 * Math.PI;
    const isCounterClockwise = false;

    if (colorStyle === ColorStyle.LINEAR) {
      gradient = ctx.createRadialGradient(
        centerX,
        centerY,
        0,
        centerX,
        centerY,
        radius
      );
      gradient.addColorStop(0, BACKDROP_COLOR);
      gradient.addColorStop(1, color);
      ctx.fillStyle = gradient;
    } else {
      ctx.fillStyle = color;
    }

    ctx.beginPath();
    ctx.arc(centerX, centerY, radius, startAngle, endAngle, isCounterClockwise);
    ctx.fill();
  } else { // Shape.RECTANGLE
    const twiceRadius = radius * 2;

    if (colorStyle === ColorStyle.LINEAR) {
      gradient = ctx.createLinearGradient(
        0,
        canvasHeight - twiceRadius,
        0,
        canvasHeight
      );
      gradient.addColorStop(0, color);
      gradient.addColorStop(1, BACKDROP_COLOR);
      ctx.fillStyle = gradient;
    } else {
      ctx.fillStyle = color;
    }

    ctx.fillRect(0, canvasHeight - twiceRadius, canvasWidth, twiceRadius);
  }
}

let frameCount = 0;
function animate(): void {
  draw();
  frameCount++;
  requestAnimationFrame(animate);
}

resizeCanvas();
requestAnimationFrame(animate);
