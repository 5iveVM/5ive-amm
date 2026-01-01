const passthrough = (value: string) => value;
const hex = () => passthrough;

const chalk = {
  bold: passthrough,
  cyan: passthrough,
  green: passthrough,
  red: passthrough,
  gray: passthrough,
  yellow: passthrough,
  dim: passthrough,
  blue: passthrough,
  magenta: passthrough,
  white: passthrough,
  yellowBright: passthrough,
  magentaBright: passthrough,
  cyanBright: passthrough,
  greenBright: passthrough,
  bgMagenta: { white: { bold: passthrough } },
  hex
};

export default chalk;
