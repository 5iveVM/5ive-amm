// Configuration types and interfaces
export type {
  FiveConfig,
  ConfigTarget,
  NetworkEndpoint,
} from './types.js';

// Configuration constants and defaults
export {
  DEFAULT_NETWORKS,
  DEFAULT_CONFIG,
  CONFIG_VALIDATORS,
} from './types.js';

// Configuration manager class and instance
export {
  ConfigManager,
  ConfigError,
  configManager,
} from './ConfigManager.js';

// Import types and classes for internal use
import type { FiveConfig } from './types.js';
import { ConfigManager } from './ConfigManager.js';

export function getConfigManager(): ConfigManager {
  return ConfigManager.getInstance();
}

export async function initConfig(): Promise<void> {
  const manager = ConfigManager.getInstance();
  await manager.init();
}

export async function getConfig(): Promise<FiveConfig> {
  const manager = ConfigManager.getInstance();
  return await manager.get();
}

export async function setConfig(updates: Partial<FiveConfig>): Promise<void> {
  const manager = ConfigManager.getInstance();
  await manager.set(updates);
}
