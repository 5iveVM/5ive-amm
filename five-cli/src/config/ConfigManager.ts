// Five CLI configuration manager.

import { promises as fs } from 'fs';
import { join } from 'path';
import { homedir } from 'os';
import type { FiveConfig, ConfigTarget } from './types.js';
import { DEFAULT_CONFIG, CONFIG_VALIDATORS } from './types.js';

export class ConfigError extends Error {
  constructor(message: string, public cause?: Error) {
    super(message);
    this.name = 'ConfigError';
  }
}

export class ConfigManager {
  private static instance: ConfigManager;
  private config: FiveConfig;
  private configPath: string;
  private initialized = false;

  private constructor() {
    this.config = { ...DEFAULT_CONFIG };
    this.configPath = this.getConfigPath();
  }

  public static getInstance(): ConfigManager {
    if (!ConfigManager.instance) {
      ConfigManager.instance = new ConfigManager();
    }
    return ConfigManager.instance;
  }

  public getConfigPath(): string {
    const xdgConfigHome = process.env.XDG_CONFIG_HOME;
    const configDir = xdgConfigHome 
      ? join(xdgConfigHome, 'five')
      : join(homedir(), '.config', 'five');
    
    return join(configDir, 'config.json');
  }

  public async init(): Promise<void> {
    try {
      const configDir = this.configPath.replace('/config.json', '');
      await fs.mkdir(configDir, { recursive: true });

      try {
        await this.load();
      } catch (error) {
        if (error instanceof ConfigError && error.message.includes('not found')) {
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

      if (!CONFIG_VALIDATORS.isValidConfig(parsedConfig)) {
        throw new ConfigError(`Invalid configuration structure in: ${this.configPath}`);
      }

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

  public async save(): Promise<void> {
    try {
      if (!CONFIG_VALIDATORS.isValidConfig(this.config)) {
        throw new ConfigError('Cannot save invalid configuration');
      }

      const configDir = this.configPath.replace('/config.json', '');
      await fs.mkdir(configDir, { recursive: true });

      const configData = JSON.stringify(this.config, null, 2);
      await fs.writeFile(this.configPath, configData, 'utf8');
    } catch (error) {
      throw new ConfigError(
        `Failed to save configuration: ${error instanceof Error ? error.message : 'Unknown error'}`,
        error instanceof Error ? error : undefined
      );
    }
  }

  public async get(): Promise<FiveConfig> {
    if (!this.initialized) {
      await this.init();
    }
    return { ...this.config };
  }

  public async set(updates: Partial<FiveConfig>): Promise<void> {
    if (!this.initialized) {
      await this.init();
    }

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

  public async getCurrentNetworkEndpoint() {
    const config = await this.get();
    return config.networks[config.target];
  }

  public async reset(): Promise<void> {
    this.config = { ...DEFAULT_CONFIG };
    await this.save();
  }

  public isInitialized(): boolean {
    return this.initialized;
  }

  public async applyOverrides(overrides: import('./types.js').ConfigOverrides): Promise<FiveConfig & { networks: any, keypairPath: string }> {
    const baseConfig = await this.get();
    
    const mergedConfig = {
      ...baseConfig,
      target: overrides.target || baseConfig.target,
      keypairPath: overrides.keypair || baseConfig.keypair || this.getDefaultKeypairPath()
    };

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
   * Set program ID for a target
   * @param programId The Five VM program ID
   * @param target The target to set it for (uses current target if not specified)
   */
  public async setProgramId(programId: string, target?: ConfigTarget): Promise<void> {
    const config = await this.get();
    const t = target || config.target;

    if (!CONFIG_VALIDATORS.isValidTarget(t)) {
      throw new ConfigError(`Invalid target: ${t}`);
    }

    const programIds = {
      ...(config.programIds || {}),
      [t]: programId
    };

    await this.set({ programIds });
  }

  /**
   * Get program ID for a target
   * @param target The target to get it for (uses current target if not specified)
   */
  public async getProgramId(target?: ConfigTarget): Promise<string | undefined> {
    const config = await this.get();
    const t = target || config.target;

    if (!CONFIG_VALIDATORS.isValidTarget(t)) {
      throw new ConfigError(`Invalid target: ${t}`);
    }

    return config.programIds?.[t];
  }

  /**
   * Clear program ID for a target
   * @param target The target to clear it for (uses current target if not specified)
   */
  public async clearProgramId(target?: ConfigTarget): Promise<void> {
    const config = await this.get();
    const t = target || config.target;

    if (!CONFIG_VALIDATORS.isValidTarget(t)) {
      throw new ConfigError(`Invalid target: ${t}`);
    }

    if (config.programIds?.[t]) {
      const programIds = { ...config.programIds };
      delete programIds[t];
      await this.set({ programIds: Object.keys(programIds).length > 0 ? programIds : undefined });
    }
  }

  /**
   * Get all program IDs
   */
  public async getAllProgramIds(): Promise<Partial<Record<ConfigTarget, string>>> {
    const config = await this.get();
    return config.programIds || {};
  }

  private getDefaultKeypairPath(): string {
    return join(homedir(), '.config', 'solana', 'id.json');
  }
}

export const configManager = ConfigManager.getInstance();
