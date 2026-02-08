export type ConfigTarget = 'wasm' | 'local' | 'devnet' | 'testnet' | 'mainnet';

export interface NetworkEndpoint {
  rpcUrl: string;
  wsUrl?: string;
}

export interface FiveConfig {
  target: ConfigTarget;
  networks: Record<ConfigTarget, NetworkEndpoint>;
  keypair?: string;
  showConfig: boolean;

  wasm?: {
    loader?: 'auto' | 'node' | 'bundler';
    modulePaths?: string[];
  };

  logging?: {
    auditLogDir?: string;
  };
}

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

export interface ConfigOverrides {
  target?: ConfigTarget;
  network?: string;
  keypair?: string;
}

export const CONFIG_VALIDATORS = {
  isValidTarget(target: string): target is ConfigTarget {
    return ['wasm', 'local', 'devnet', 'testnet', 'mainnet'].includes(target);
  },

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
