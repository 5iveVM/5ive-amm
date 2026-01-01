/**
 * Five CLI Configuration System
 * 
 * Exports all configuration-related functionality for the Five CLI.
 * Provides types, configuration manager, and singleton instance access.
 */

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

/**
 * Convenience function to get the configuration manager instance
 * This is the recommended way to access configuration throughout the CLI
 * 
 * @returns The singleton ConfigManager instance
 * 
 * @example
 * ```typescript
 * import { getConfigManager } from './config';
 * 
 * const config = getConfigManager();
 * const currentTarget = await config.getTarget();
 * ```
 */
export function getConfigManager(): ConfigManager {
  return ConfigManager.getInstance();
}

/**
 * Convenience function to initialize the configuration system
 * Should be called early in CLI startup to ensure config is ready
 * 
 * @returns Promise that resolves when configuration is initialized
 * 
 * @example
 * ```typescript
 * import { initConfig } from './config';
 * 
 * // In your main CLI entry point
 * await initConfig();
 * ```
 */
export async function initConfig(): Promise<void> {
  const manager = ConfigManager.getInstance();
  await manager.init();
}

/**
 * Convenience function to get the current configuration
 * Automatically initializes the config manager if needed
 * 
 * @returns Promise that resolves to the current configuration
 * 
 * @example
 * ```typescript
 * import { getConfig } from './config';
 * 
 * const config = await getConfig();
 * console.log(`Current target: ${config.target}`);
 * ```
 */
export async function getConfig(): Promise<FiveConfig> {
  const manager = ConfigManager.getInstance();
  return await manager.get();
}

/**
 * Convenience function to update configuration
 * Automatically initializes the config manager if needed
 * 
 * @param updates Partial configuration updates to apply
 * 
 * @example
 * ```typescript
 * import { setConfig } from './config';
 * 
 * // Switch to mainnet
 * await setConfig({ target: 'mainnet' });
 * 
 * // Update keypair path
 * await setConfig({ keypair: '/path/to/keypair.json' });
 * ```
 */
export async function setConfig(updates: Partial<FiveConfig>): Promise<void> {
  const manager = ConfigManager.getInstance();
  await manager.set(updates);
}