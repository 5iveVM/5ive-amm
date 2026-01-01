type Spinner = {
  text?: string;
  start: (text?: string) => Spinner;
  succeed: (text?: string) => Spinner;
  fail: (text?: string) => Spinner;
  warn: (text?: string) => Spinner;
  info: (text?: string) => Spinner;
  stop: () => Spinner;
  stopAndPersist: () => Spinner;
  clear: () => Spinner;
};

const createSpinner = (text?: string): Spinner => {
  const spinner: Spinner = {
    text,
    start(newText?: string) {
      spinner.text = newText ?? spinner.text;
      return spinner;
    },
    succeed(newText?: string) {
      spinner.text = newText ?? spinner.text;
      return spinner;
    },
    fail(newText?: string) {
      spinner.text = newText ?? spinner.text;
      return spinner;
    },
    warn(newText?: string) {
      spinner.text = newText ?? spinner.text;
      return spinner;
    },
    info(newText?: string) {
      spinner.text = newText ?? spinner.text;
      return spinner;
    },
    stop() {
      return spinner;
    },
    stopAndPersist() {
      return spinner;
    },
    clear() {
      return spinner;
    }
  };

  return spinner;
};

const ora = (text?: string) => createSpinner(text);

export default ora;
