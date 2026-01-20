/**
 * Network configuration for Five VM deployments
 * Supports localnet and devnet with configurable endpoints and program IDs
 */

export type NetworkType = 'localnet' | 'devnet';

export interface NetworkConfig {
    name: string;
    rpcUrl: string;
    programId: string;
    explorerUrl?: string;
}

export const NETWORKS: Record<NetworkType, NetworkConfig> = {
    localnet: {
        name: 'Localnet',
        rpcUrl: 'http://127.0.0.1:8899',
        programId: '2DXiYbzfSMwkDSxc9aWEaW7XgJjkNzGdADfRN4FbxMNN',
        explorerUrl: undefined // No explorer for localnet
    },
    devnet: {
        name: 'Devnet',
        rpcUrl: 'https://api.devnet.solana.com',
        programId: '2DXiYbzfSMwkDSxc9aWEaW7XgJjkNzGdADfRN4FbxMNN', // Placeholder - update when deployed
        explorerUrl: 'https://explorer.solana.com'
    }
} as const;

/**
 * Get explorer URL for a transaction or account
 */
export function getExplorerUrl(
    network: NetworkType,
    type: 'tx' | 'address',
    value: string
): string | null {
    const config = NETWORKS[network];
    if (!config.explorerUrl) return null;

    const cluster = network === 'localnet' ? '' : `?cluster=${network}`;
    return `${config.explorerUrl}/${type}/${value}${cluster}`;
}

/**
 * Get the RPC URL for a network
 */
export function getRpcUrl(network: NetworkType): string {
    return NETWORKS[network].rpcUrl;
}

/**
 * Get the Five VM Program ID for a network
 */
export function getProgramId(network: NetworkType): string {
    return NETWORKS[network].programId;
}
