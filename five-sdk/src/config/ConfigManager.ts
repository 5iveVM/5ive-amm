/**
 * Five SDK Configuration Manager (Simplified for SDK)
 * 
 * Provides configuration access for WASM loading and other SDK settings.
 * Client-agnostic implementation.
 */

import { FiveSDKConfig } from '../types.js';

const DEFAULT_CONFIG: FiveSDKConfig = {
    network: 'devnet',
    debug: false
};

export class ConfigManager {
    private static instance: ConfigManager;
    private config: FiveSDKConfig;

    private constructor() {
        this.config = { ...DEFAULT_CONFIG };
    }

    public static getInstance(): ConfigManager {
        if (!ConfigManager.instance) {
            ConfigManager.instance = new ConfigManager();
        }
        return ConfigManager.instance;
    }

    public async get(): Promise<any> {
        return this.config;
    }

    public getSync(): any {
        return this.config;
    }

    public async set(config: Partial<FiveSDKConfig>): Promise<void> {
        this.config = { ...this.config, ...config };
    }
}
