/**
 * Advanced Logger for Five CLI
 * 
 * Provides structured logging with performance monitoring, context tracking,
 * and configurable output formats for optimal development and debugging experience.
 */

import { writeFile, appendFile, mkdir } from 'fs/promises';
import { join, dirname } from 'path';
import { LogLevel, Logger } from '../types.js';
import chalk from 'chalk';
import { palette } from './cli-ui.js';

export interface LoggerOptions {
  level: LogLevel;
  enableColors: boolean;
  enableTimestamps: boolean;
  enableContext: boolean;
  logFile?: string;
  maxLogSize?: number;
  enablePerformance?: boolean;
}

export class FiveLogger implements Logger {
  private options: LoggerOptions;
  private context: Map<string, any> = new Map();
  private performanceMarks: Map<string, number> = new Map();

  constructor(options: Partial<LoggerOptions> = {}) {
    this.options = {
      level: 'info',
      enableColors: true,
      enableTimestamps: true,
      enableContext: false,
      maxLogSize: 10 * 1024 * 1024, // 10MB
      enablePerformance: false,
      ...options
    };
  }

  /**
   * Debug level logging - for detailed diagnostic information
   */
  debug(message: string, ...args: any[]): void {
    if (this.shouldLog('debug')) {
      this.log('debug', message, args);
    }
  }

  /**
   * Info level logging - for general information
   */
  info(message: string, ...args: any[]): void {
    if (this.shouldLog('info')) {
      this.log('info', message, args);
    }
  }

  /**
   * Warning level logging - for potentially harmful situations
   */
  warn(message: string, ...args: any[]): void {
    if (this.shouldLog('warn')) {
      this.log('warn', message, args);
    }
  }

  /**
   * Error level logging - for error events
   */
  error(message: string, ...args: any[]): void {
    if (this.shouldLog('error')) {
      this.log('error', message, args);
    }
  }

  /**
   * Log with performance measurement
   */
  perf(operation: string, message: string, ...args: any[]): void {
    if (this.options.enablePerformance) {
      const duration = this.endPerformanceMark(operation);
      const perfMessage = duration !== null
        ? `${message} (${duration}ms)`
        : message;
      this.info(`[PERF] ${perfMessage}`, ...args);
    }
  }

  /**
   * Start performance measurement
   */
  startPerformanceMark(operation: string): void {
    if (this.options.enablePerformance) {
      this.performanceMarks.set(operation, Date.now());
    }
  }

  /**
   * End performance measurement and return duration
   */
  endPerformanceMark(operation: string): number | null {
    if (!this.options.enablePerformance) {
      return null;
    }

    const startTime = this.performanceMarks.get(operation);
    if (startTime) {
      const duration = Date.now() - startTime;
      this.performanceMarks.delete(operation);
      return duration;
    }
    return null;
  }

  /**
   * Add context information to all subsequent logs
   */
  addContext(key: string, value: any): void {
    this.context.set(key, value);
  }

  /**
   * Remove context information
   */
  removeContext(key: string): void {
    this.context.delete(key);
  }

  /**
   * Clear all context information
   */
  clearContext(): void {
    this.context.clear();
  }

  /**
   * Create a child logger with additional context
   */
  child(context: Record<string, any>): FiveLogger {
    const childLogger = new FiveLogger(this.options);

    // Copy parent context
    for (const [key, value] of this.context) {
      childLogger.addContext(key, value);
    }

    // Add child context
    for (const [key, value] of Object.entries(context)) {
      childLogger.addContext(key, value);
    }

    return childLogger;
  }

  /**
   * Log structured data (useful for debugging complex objects)
   */
  logObject(level: LogLevel, object: any, label?: string): void {
    if (this.shouldLog(level)) {
      const message = label ? `${label}:` : 'Object:';
      this.log(level, message, [JSON.stringify(object, null, 2)]);
    }
  }

  /**
   * Log with specific formatting for CLI operations
   */
  command(command: string, args: string[], duration?: number): void {
    const formattedCommand = `${command} ${args.join(' ')}`;
    const message = duration
      ? `Command executed: ${formattedCommand} (${duration}ms)`
      : `Executing command: ${formattedCommand}`;
    this.info(message);
  }

  /**
   * Log compilation results with formatted output
   */
  compilation(
    sourceFile: string,
    success: boolean,
    duration: number,
    details?: any
  ): void {
    const status = success ? 'SUCCESS' : 'FAILED';
    const color = success ? chalk.green : chalk.red;

    if (this.options.enableColors) {
      console.log(color(`[COMPILE ${status}] ${sourceFile} (${duration}ms)`));
    } else {
      console.log(`[COMPILE ${status}] ${sourceFile} (${duration}ms)`);
    }

    if (details && this.shouldLog('debug')) {
      this.logObject('debug', details, 'Compilation details');
    }
  }

  /**
   * Log VM execution results
   */
  execution(
    success: boolean,
    computeUnits: number,
    duration: number,
    details?: any
  ): void {
    const status = success ? 'SUCCESS' : 'FAILED';
    const color = success ? chalk.green : chalk.red;

    const message = `[VM ${status}] CU: ${computeUnits}, Time: ${duration}ms`;

    if (this.options.enableColors) {
      console.log(color(message));
    } else {
      console.log(message);
    }

    if (details && this.shouldLog('debug')) {
      this.logObject('debug', details, 'Execution details');
    }
  }

  /**
   * Core logging method
   */
  private log(level: LogLevel, message: string, args: any[]): void {
    const logEntry = this.formatLogEntry(level, message, args);

    // Output to console
    this.outputToConsole(level, logEntry);

    // Output to file if configured
    if (this.options.logFile) {
      this.outputToFile(logEntry).catch(err => {
        console.error('Failed to write to log file:', err);
      });
    }
  }

  /**
   * Format log entry with timestamp, level, context, and message
   */
  private formatLogEntry(level: LogLevel, message: string, args: any[]): string {
    const parts: string[] = [];

    // Timestamp
    if (this.options.enableTimestamps) {
      const timestamp = new Date().toISOString();
      parts.push(`[${timestamp}]`);
    }

    // Log level
    parts.push(`[${level.toUpperCase()}]`);

    // Context
    if (this.options.enableContext && this.context.size > 0) {
      const contextStr = Array.from(this.context.entries())
        .map(([key, value]) => `${key}=${value}`)
        .join(',');
      parts.push(`[${contextStr}]`);
    }

    // Message
    parts.push(message);

    // Arguments
    if (args.length > 0) {
      const argsStr = args.map(arg =>
        typeof arg === 'object' ? JSON.stringify(arg) : String(arg)
      ).join(' ');
      parts.push(argsStr);
    }

    return parts.join(' ');
  }

  /**
   * Output log entry to console with appropriate colors
   */
  private outputToConsole(level: LogLevel, logEntry: string): void {
    if (!this.options.enableColors) {
      console.log(logEntry);
      return;
    }

    switch (level) {
      case 'debug':
        console.log(chalk.gray(logEntry));
        break;
      case 'info':
        console.log(chalk.cyan(logEntry));
        break;
      case 'warn':
        console.log(chalk.yellow(logEntry));
        break;
      case 'error':
        console.error(chalk.red(logEntry));
        break;
      default:
        console.log(logEntry);
    }
  }

  /**
   * Output log entry to file
   */
  private async outputToFile(logEntry: string): Promise<void> {
    if (!this.options.logFile) {
      return;
    }

    try {
      // Ensure log directory exists
      const logDir = dirname(this.options.logFile);
      await mkdir(logDir, { recursive: true });

      // Append to log file
      await appendFile(this.options.logFile, logEntry + '\n');

      // TODO: Implement log rotation based on maxLogSize

    } catch (error) {
      // Silent fail to avoid infinite logging loops
    }
  }

  /**
   * Check if message should be logged based on current log level
   */
  private shouldLog(messageLevel: LogLevel): boolean {
    const levels: Record<LogLevel, number> = {
      debug: 0,
      info: 1,
      warn: 2,
      error: 3
    };

    return levels[messageLevel] >= levels[this.options.level];
  }

  /**
   * Update logger options
   */
  updateOptions(options: Partial<LoggerOptions>): void {
    this.options = { ...this.options, ...options };
  }

  /**
   * Get current logger configuration
   */
  getOptions(): LoggerOptions {
    return { ...this.options };
  }
}

/**
 * Create a logger instance with Five CLI defaults
 */
export function createLogger(options: Partial<LoggerOptions> = {}): FiveLogger {
  const defaultOptions: Partial<LoggerOptions> = {
    level: process.env.DEBUG ? 'debug' : 'info',
    enableColors: process.stdout.isTTY,
    enableTimestamps: false,
    enableContext: !!process.env.DEBUG,
    enablePerformance: !!process.env.PERF
  };

  return new FiveLogger({ ...defaultOptions, ...options });
}

/**
 * Create a file logger for persistent logging
 */
export function createFileLogger(
  logFile: string,
  options: Partial<LoggerOptions> = {}
): FiveLogger {
  return new FiveLogger({
    ...options,
    logFile,
    enableColors: false, // File output shouldn't have ANSI colors
    enableTimestamps: true
  });
}

/**
 * Utility function to measure and log function execution time
 */
export function measureTime<T>(
  logger: FiveLogger,
  operation: string,
  fn: () => T | Promise<T>
): T | Promise<T> {
  logger.startPerformanceMark(operation);

  const result = fn();

  if (result instanceof Promise) {
    return result.finally(() => {
      logger.perf(operation, `Completed ${operation}`);
    });
  } else {
    logger.perf(operation, `Completed ${operation}`);
    return result;
  }
}
