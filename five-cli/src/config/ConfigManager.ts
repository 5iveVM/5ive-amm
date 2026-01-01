/**
 * Five CLI Configuration Manager
 * 
 * Handles configuration file loading, saving, and management with XDG directory support.
 * Implements singleton pattern for global config access throughout the CLI.
 */

import { promises as fs } from 'fs';
import { join } from 'path';
import { homedir } from 'os';
import type { FiveConfig, ConfigTarget } from './types.js';
import { DEFAULT_CONFIG, CONFIG_VALIDATORS } from './types.js';

/**
 * Configuration management errors
 */
export class ConfigError extends Error {
  constructor(message: string, public cause?: Error) {
    super(message);
    this.name = 'ConfigError';
  }
}

/**
 * Five CLI Configuration Manager
 * 
 * Manages configuration file operations with XDG Base Directory support.
 * Provides a singleton interface for configuration access across the CLI.
 */
export class ConfigManager {
  private static instance: ConfigManager;
  private config: FiveConfig;
  private configPath: string;
  private initialized = false;

  /**
   * Private constructor for singleton pattern
   */
  private constructor() {
    this.config = { ...DEFAULT_CONFIG };
    this.configPath = this.getConfigPath();
  }

  /**
   * Get the singleton ConfigManager instance
   */
  public static getInstance(): ConfigManager {
    if (!ConfigManager.instance) {
      ConfigManager.instance = new ConfigManager();
    }
    return ConfigManager.instance;
  }

  /**
   * Get the XDG-compliant config file path
   * Follows XDG Base Directory specification:
   * - Uses $XDG_CONFIG_HOME if set
   * - Falls back to ~/.config/five/config.json
   */
  public getConfigPath(): string {
    const xdgConfigHome = process.env.XDG_CONFIG_HOME;
    const configDir = xdgConfigHome 
      ? join(xdgConfigHome, 'five')
      : join(homedir(), '.config', 'five');
    
    return join(configDir, 'config.json');
  }

  /**
   * Initialize the configuration system
   * Creates config directory and default config file if they don't exist
   */
  public async init(): Promise<void> {
    try {
      // Ensure config directory exists
      const configDir = this.configPath.replace('/config.json', '');
      await fs.mkdir(configDir, { recursive: true });

      // Try to load existing config, create default if none exists
      try {
        await this.load();
      } catch (error) {
        if (error instanceof ConfigError && error.message.includes('not found')) {
          // Create default config file
          await this.save();
        } else {
          throw error;
        }
      }

      this.initialized = true;
    } catch (error) {
      throw new ConfigError(
        `Failed to initialize configuration: ${error instanceof Error ? error.message : 'Unknown error'}`,
        error instanceof Error ? error : undefined
      );
    }
  }

  /**
   * Load configuration from file
   * @throws ConfigError if file doesn't exist or is invalid
   */
  public async load(): Promise<FiveConfig> {
    try {
      const configData = await fs.readFile(this.configPath, 'utf8');
      
      let parsedConfig: any;
      try {
        parsedConfig = JSON.parse(configData);
      } catch (parseError) {
        throw new ConfigError(
          `Invalid JSON in config file: ${this.configPath}`,
          parseError instanceof Error ? parseError : undefined
        );
      }

      // Validate the parsed configuration
      if (!CONFIG_VALIDATORS.isValidConfig(parsedConfig)) {
        throw new ConfigError(`Invalid configuration structure in: ${this.configPath}`);
      }

      // Merge with defaults to ensure all required fields are present
      this.config = {
        ...DEFAULT_CONFIG,
        ...parsedConfig,
        networks: {
          ...DEFAULT_CONFIG.networks,
          ...parsedConfig.networks,
        },
      };

      return this.config;
    } catch (error) {
      if (error instanceof ConfigError) {
        throw error;
      }

      if ((error as any)?.code === 'ENOENT') {
        throw new ConfigError(`Config file not found: ${this.configPath}`);
      }

      throw new ConfigError(
        `Failed to load configuration: ${error instanceof Error ? error.message : 'Unknown error'}`,
        error instanceof Error ? error : undefined
      );
    }
  }

  /**
   * Save current configuration to file
   * @throws ConfigError if save operation fails
   */
  public async save(): Promise<void> {
    try {
      // Validate configuration before saving
      if (!CONFIG_VALIDATORS.isValidConfig(this.config)) {
        throw new ConfigError('Cannot save invalid configuration');
      }

      // Ensure config directory exists
      const configDir = this.configPath.replace('/config.json', '');
      await fs.mkdir(configDir, { recursive: true });

      // Write configuration with pretty formatting
      const configData = JSON.stringify(this.config, null, 2);
      await fs.writeFile(this.configPath, configData, 'utf8');
    } catch (error) {
      throw new ConfigError(
        `Failed to save configuration: ${error instanceof Error ? error.message : 'Unknown error'}`,
        error instanceof Error ? error : undefined
      );
    }
  }

  /**
   * Get the current configuration
   * Ensures the config manager is initialized
   */
  public async get(): Promise<FiveConfig> {
    if (!this.initialized) {
      await this.init();
    }
    return { ...this.config };
  }

  /**
   * Update configuration with partial values
   * @param updates Partial configuration updates
   */
  public async set(updates: Partial<FiveConfig>): Promise<void> {
    if (!this.initialized) {
      await this.init();
    }

    // Create updated configuration
    const updatedConfig = {
      ...this.config,
      ...updates,
    };

    // If networks are being updated, merge with existing networks
    if (updates.networks) {
      updatedConfig.networks = {
        ...this.config.networks,
        ...updates.networks,
      };
    }

    // Validate the updated configuration
    if (!CONFIG_VALIDATORS.isValidConfig(updatedConfig)) {
      throw new ConfigError('Invalid configuration update');
    }

    this.config = updatedConfig;
    await this.save();
  }

  /**
   * Set the target network
   * @param target The target network to set
   */
  public async setTarget(target: ConfigTarget): Promise<void> {
    if (!CONFIG_VALIDATORS.isValidTarget(target)) {
      throw new ConfigError(`Invalid target: ${target}`);
    }
    
    await this.set({ target });
  }

  /**
   * Set the keypair file path
   * @param keypairPath Path to the keypair file
   */
  public async setKeypair(keypairPath: string): Promise<void> {
    await this.set({ keypair: keypairPath });
  }


  /**
   * Toggle config display in command output
   * @param show Whether to show config details
   */
  public async setShowConfig(show: boolean): Promise<void> {
    await this.set({ showConfig: show });
  }

  /**
   * Get the current target network
   */
  public async getTarget(): Promise<ConfigTarget> {
    const config = await this.get();
    return config.target;
  }

  /**
   * Get the current network endpoint configuration
   */
  public async getCurrentNetworkEndpoint() {
    const config = await this.get();
    return config.networks[config.target];
  }

  /**
   * Reset configuration to defaults
   */
  public async reset(): Promise<void> {
    this.config = { ...DEFAULT_CONFIG };
    await this.save();
  }

  /**
   * Check if the configuration is properly initialized
   */
  public isInitialized(): boolean {
    return this.initialized;
  }

  /**
   * Apply configuration overrides and return merged config
   * @param overrides CLI option overrides
   * @returns Merged configuration with overrides applied
   */
  public async applyOverrides(overrides: import('./types.js').ConfigOverrides): Promise<FiveConfig & { networks: any, keypairPath: string }> {
    const baseConfig = await this.get();
    
    // Create merged configuration
    const mergedConfig = {
      ...baseConfig,
      target: overrides.target || baseConfig.target,
      keypairPath: overrides.keypair || baseConfig.keypair || this.getDefaultKeypairPath()
    };

    // Handle network override - if provided, create custom network config
    if (overrides.network) {
      mergedConfig.networks = {
        ...baseConfig.networks,
        [mergedConfig.target]: {
          ...baseConfig.networks[mergedConfig.target],
          rpcUrl: overrides.network
        }
      };
    }

    return mergedConfig;
  }

  /**
   * Get target context prefix for display
   * @param target The target network
   * @returns Formatted target prefix like '[devnet]'
   */
  public static getTargetPrefix(target: import('./types.js').ConfigTarget): string {
    const colors = {
      wasm: '\x1b[36m',     // cyan
      local: '\x1b[90m',    // gray
      devnet: '\x1b[33m',   // yellow
      testnet: '\x1b[35m',  // magenta  
      mainnet: '\x1b[31m'   // red
    };
    
    const reset = '\x1b[0m';
    const color = colors[target] || colors.devnet;
    
    return `${color}[${target}]${reset}`;
  }

  /**
   * Get default keypair path following Solana CLI conventions
   */
  private getDefaultKeypairPath(): string {
    return join(homedir(), '.config', 'solana', 'id.json');
  }
}

/**
 * Singleton instance export for convenient access
 */
export const configManager = ConfigManager.getInstance();