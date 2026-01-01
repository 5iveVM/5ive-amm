/**
 * Minimal retro CLI UI helpers using Five frontend colors (Rose Pine).
 */

import chalk from 'chalk';

export const palette = {
  base: '#232136',
  surface: '#2a273f',
  overlay: '#393552',
  muted: '#6e6a86',
  subtle: '#908caa',
  text: '#e0def4',
  love: '#eb6f92',
  gold: '#f6c177',
  rose: '#ea9a97',
  pine: '#3e8fb0',
  foam: '#9ccfd8',
  iris: '#c4a7e7'
};

const colors = {
  text: chalk.white,
  muted: chalk.gray,
  subtle: chalk.gray,
  accent: chalk.magenta,
  success: chalk.green,
  warn: chalk.yellow,
  error: chalk.red,
  info: chalk.cyan,
  rose: chalk.magentaBright
};

export const uiColors = {
  text: (value: string) => colors.text(value),
  muted: (value: string) => colors.muted(value),
  subtle: (value: string) => colors.subtle(value),
  accent: (value: string) => colors.accent(value),
  success: (value: string) => colors.success(value),
  warn: (value: string) => colors.warn(value),
  error: (value: string) => colors.error(value),
  info: (value: string) => colors.info(value),
  rose: (value: string) => colors.rose(value)
};

export function brandLine(): string {
  return `${colors.rose('==')} ${colors.accent('5IVE CLI')} ${colors.rose('==')}`;
}

export function section(title: string): string {
  return colors.subtle(title.toUpperCase());
}

export function commandExample(command: string, description: string): string {
  return `  ${colors.info(command)} ${colors.muted(description)}`;
}

export function keyValue(key: string, value: string): string {
  return `  ${colors.info(key)} ${colors.text(value)}`;
}

export function success(message: string): string {
  return `${colors.success('OK')} ${colors.text(message)}`;
}

export function warning(message: string): string {
  return `${colors.warn('warn:')} ${colors.text(message)}`;
}

export function error(message: string): string {
  return `${colors.error('error:')} ${colors.text(message)}`;
}

export function hint(message: string): string {
  return `${colors.muted('hint:')} ${colors.muted(message)}`;
}

export function commandNotFound(command: string, suggestions: string[]): string {
  const parts = [error(`unknown command "${command}"`)];
  if (suggestions.length > 0) {
    parts.push(hint(`did you mean ${suggestions.join(', ')}?`));
  }
  return parts.join('\n');
}

export function isTTY(): boolean {
  return Boolean(process.stdout.isTTY);
}
