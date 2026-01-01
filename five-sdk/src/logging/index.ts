/**
 * Simple logging that can be silenced in production
 */

const isProduction = (typeof process !== 'undefined' && process?.env?.NODE_ENV === 'production');

export const log = {
  debug: (message: string, ...args: any[]) => !isProduction && console.debug(`[FiveSDK] ${message}`, ...args),
  info: (message: string, ...args: any[]) => !isProduction && console.info(`[FiveSDK] ${message}`, ...args), 
  warn: (message: string, ...args: any[]) => console.warn(`[FiveSDK] ${message}`, ...args),
  error: (message: string, ...args: any[]) => console.error(`[FiveSDK] ${message}`, ...args)
};