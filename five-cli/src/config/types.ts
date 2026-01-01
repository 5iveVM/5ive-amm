/**
 * Five CLI Configuration Types
 * 
 * Defines the configuration structure and types for the Five CLI.
 * Supports multiple network targets and configurable deployment settings.
 */

/**
 * Supported Five network targets
 */
export type ConfigTarget = 'wasm' | 'local' | 'devnet' | 'testnet' | 'mainnet';

/**
 * Network endpoint configuration mapping
 */
export interface NetworkEndpoint {
  rpcUrl: string;
  wsUrl?: string;
}

/**
 * Complete Five CLI configuration interface
 */
export interface FiveConfig {
  /** Current deployment target */
  target: ConfigTarget;
  
  /** Network endpoint configurations */
  networks: Record<ConfigTarget, NetworkEndpoint>;
  
  /** Path to keypair file for transactions */
  keypair?: string;
  
  /** Whether to show config details in command output */
  showConfig: boolean;

  /** Optional WASM loader configuration */
  wasm?: {
    /** Loader preference: auto (default), node, bundler */
    loader?: 'auto' | 'node' | 'bundler';
    /** Explicit module candidate paths (JS modules), absolute or relative */
    modulePaths?: string[];
  };

  /** Optional logging configuration */
  logging?: {
    /** Audit log directory for on-chain operations */
    auditLogDir?: string;
  };
}

/**
 * Default network endpoint configurations
 */
export const DEFAULT_NETWORKS: Record<ConfigTarget, NetworkEndpoint> = {
  wasm: {
    rpcUrl: 'wasm://local-execution',
    wsUrl: undefined,
  },
  local: {
    rpcUrl: 'http://127.0.0.1:8899',
    wsUrl: 'ws://127.0.0.1:8900',
  },
  devnet: {
    rpcUrl: 'https://api.devnet.solana.com',
    wsUrl: 'wss://api.devnet.solana.com',
  },
  testnet: {
    rpcUrl: 'https://api.testnet.solana.com',
    wsUrl: 'wss://api.testnet.solana.com',
  },
  mainnet: {
    rpcUrl: 'https://api.mainnet-beta.solana.com',
    wsUrl: 'wss://api.mainnet-beta.solana.com',
  },
};

/**
 * Default Five CLI configuration
 * Uses devnet as the default target for safety
 */
export const DEFAULT_CONFIG: FiveConfig = {
  target: 'devnet',
  networks: DEFAULT_NETWORKS,
  showConfig: false,
  wasm: {
    loader: 'auto',
    modulePaths: [],
  },
  logging: {}
};

/**
 * Configuration override options for CLI commands
 */
export interface ConfigOverrides {
  /** Override target network */
  target?: ConfigTarget;
  /** Override network RPC URL */
  network?: string;
  /** Override keypair file path */
  keypair?: string;
}

/**
 * Configuration validation utilities
 */
export const CONFIG_VALIDATORS = {
  /**
   * Validates if a target is supported
   */
  isValidTarget(target: string): target is ConfigTarget {
    return ['wasm', 'local', 'devnet', 'testnet', 'mainnet'].includes(target);
  },

  /**
   * Validates if a network endpoint configuration is valid
   */
  isValidNetworkEndpoint(endpoint: any): endpoint is NetworkEndpoint {
    return (
      typeof endpoint === 'object' &&
      endpoint !== null &&
      typeof endpoint.rpcUrl === 'string' &&
      endpoint.rpcUrl.length > 0 &&
      (endpoint.wsUrl === undefined || typeof endpoint.wsUrl === 'string')
    );
  },

  /**
   * Validates if a complete config object is valid
   */
  isValidConfig(config: any): config is FiveConfig {
    if (typeof config !== 'object' || config === null) {
      return false;
    }

    // Validate target
    if (!this.isValidTarget(config.target)) {
      return false;
    }

    // Validate networks
    if (typeof config.networks !== 'object' || config.networks === null) {
      return false;
    }

    for (const [target, endpoint] of Object.entries(config.networks)) {
      if (!this.isValidTarget(target) || !this.isValidNetworkEndpoint(endpoint)) {
        return false;
      }
    }

    // Validate optional fields
    if (config.keypair !== undefined && typeof config.keypair !== 'string') {
      return false;
    }

    // Optional wasm config
    if (config.wasm !== undefined) {
      if (typeof config.wasm !== 'object' || config.wasm === null) return false;
      if (config.wasm.loader !== undefined && !['auto','node','bundler'].includes(config.wasm.loader)) return false;
      if (config.wasm.modulePaths !== undefined) {
        if (!Array.isArray(config.wasm.modulePaths)) return false;
        if (!config.wasm.modulePaths.every((p: any) => typeof p === 'string')) return false;
      }
    }

    // Optional logging
    if (config.logging !== undefined) {
      if (typeof config.logging !== 'object' || config.logging === null) return false;
      if (config.logging.auditLogDir !== undefined && typeof config.logging.auditLogDir !== 'string') return false;
    }

    if (typeof config.showConfig !== 'boolean') {
      return false;
    }

    return true;
  },
};
