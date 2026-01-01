/**
 * Simple logging that can be silenced in production
 */
const isProduction = (typeof process !== 'undefined' && process?.env?.NODE_ENV === 'production');
export const log = {
    debug: (message, ...args) => !isProduction && console.debug(`[FiveSDK] ${message}`, ...args),
    info: (message, ...args) => !isProduction && console.info(`[FiveSDK] ${message}`, ...args),
    warn: (message, ...args) => console.warn(`[FiveSDK] ${message}`, ...args),
    error: (message, ...args) => console.error(`[FiveSDK] ${message}`, ...args)
};
//# sourceMappingURL=index.js.map