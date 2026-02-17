const SOURCE_IMPORT_ERROR = [
  "[FiveSDK] Unsupported runtime import: `five-sdk/src/FiveSDK.js`.",
  "This source path is legacy and can emit stale instruction encoding.",
  "Use `@5ive-tech/sdk` or `five-sdk/dist/index.js` instead.",
].join(" ");

throw new Error(SOURCE_IMPORT_ERROR);
