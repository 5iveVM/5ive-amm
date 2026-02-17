const SOURCE_IMPORT_ERROR = [
  "[FiveSDK] Unsupported runtime import: `five-sdk/src/index.js`.",
  "This source entrypoint is legacy and can bypass canonical execute encoding.",
  "Use `@5ive-tech/sdk` or `five-sdk/dist/index.js` instead.",
].join(" ");

throw new Error(SOURCE_IMPORT_ERROR);
