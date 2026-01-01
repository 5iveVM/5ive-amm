/**
 * Five SDK Script Metadata System
 *
 * Real implementation for parsing script account data and extracting ABI information
 * from deployed Five scripts. This replaces mock implementations with production-ready
 * Solana account data parsing.
 */
import { Base58Utils } from '../crypto/index.js';
import { normalizeAbiFunctions } from '../utils/abi.js';
/**
 * Script metadata parser and manager
 */
export class ScriptMetadataParser {
    static SCRIPT_MAGIC = new Uint8Array([
        0x46, 0x49, 0x56, 0x45, 0x5F, 0x53, 0x43, 0x52 // "FIVE_SCR"
    ]);
    static CURRENT_VERSION = 1;
    static HEADER_SIZE = 64; // Fixed header size
    /**
     * Parse script metadata from account data
     */
    static parseMetadata(accountData, address) {
        if (accountData.length < this.HEADER_SIZE) {
            throw new Error(`Invalid script account: data too small (${accountData.length} bytes, minimum ${this.HEADER_SIZE})`);
        }
        let offset = 0;
        // Parse header
        const magic = accountData.slice(offset, offset + 8);
        offset += 8;
        if (!this.arraysEqual(magic, this.SCRIPT_MAGIC)) {
            throw new Error('Invalid script account: magic bytes mismatch');
        }
        const version = this.readU32(accountData, offset);
        offset += 4;
        if (version > this.CURRENT_VERSION) {
            throw new Error(`Unsupported script version: ${version} (max supported: ${this.CURRENT_VERSION})`);
        }
        const timestamp = this.readU64(accountData, offset);
        offset += 8;
        const authority = accountData.slice(offset, offset + 32);
        offset += 32;
        const bytecodeLength = this.readU32(accountData, offset);
        offset += 4;
        const abiLength = this.readU32(accountData, offset);
        offset += 4;
        // Skip reserved space
        offset += 8;
        // Validate data lengths
        const expectedSize = this.HEADER_SIZE + bytecodeLength + abiLength;
        if (accountData.length < expectedSize) {
            throw new Error(`Invalid script account: expected ${expectedSize} bytes, got ${accountData.length}`);
        }
        // Extract bytecode
        const bytecode = accountData.slice(offset, offset + bytecodeLength);
        offset += bytecodeLength;
        // Extract and parse ABI
        const abiData = accountData.slice(offset, offset + abiLength);
        const abiJson = new TextDecoder().decode(abiData);
        let abi;
        try {
            abi = JSON.parse(abiJson);
        }
        catch (error) {
            throw new Error(`Invalid ABI JSON: ${error instanceof Error ? error.message : 'Parse error'}`);
        }
        return {
            address,
            bytecode,
            abi,
            deployedAt: timestamp,
            version: version.toString(),
            authority: Base58Utils.encode(authority)
        };
    }
    /**
     * Get script metadata from blockchain using account fetcher
     */
    static async getScriptMetadata(accountFetcher, scriptAddress) {
        try {
            // Validate address format (basic base58 check)
            if (!scriptAddress || scriptAddress.length < 32 || scriptAddress.length > 44) {
                throw new Error(`Invalid script address format: ${scriptAddress}`);
            }
            // Fetch account data
            const accountData = await accountFetcher.getAccountData(scriptAddress);
            if (!accountData) {
                throw new Error(`Script account not found: ${scriptAddress}`);
            }
            if (!accountData.data || accountData.data.length === 0) {
                throw new Error(`Script account has no data: ${scriptAddress}`);
            }
            // Parse metadata
            return this.parseMetadata(accountData.data, scriptAddress);
        }
        catch (error) {
            throw new Error(`Failed to get script metadata for ${scriptAddress}: ${error instanceof Error ? error.message : 'Unknown error'}`);
        }
    }
    /**
     * Get multiple script metadata entries using account fetcher
     */
    static async getMultipleScriptMetadata(accountFetcher, scriptAddresses) {
        const results = new Map();
        // Validate addresses (basic format check)
        const validAddresses = [];
        for (const address of scriptAddresses) {
            if (address && address.length >= 32 && address.length <= 44) {
                validAddresses.push(address);
            }
            else {
                results.set(address, null);
            }
        }
        if (validAddresses.length === 0) {
            return results;
        }
        try {
            // Batch fetch account data
            const accountDataMap = await accountFetcher.getMultipleAccountsData(validAddresses);
            // Parse metadata for each account
            for (const address of validAddresses) {
                const accountData = accountDataMap.get(address);
                if (!accountData || !accountData.data || accountData.data.length === 0) {
                    results.set(address, null);
                    continue;
                }
                try {
                    const metadata = this.parseMetadata(accountData.data, address);
                    results.set(address, metadata);
                }
                catch (error) {
                    console.warn(`Failed to parse metadata for ${address}:`, error);
                    results.set(address, null);
                }
            }
        }
        catch (error) {
            throw new Error(`Batch metadata fetch failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
        }
        return results;
    }
    /**
     * Extract function signatures from ABI
     */
    static extractFunctionSignatures(abi) {
        const functions = normalizeAbiFunctions(abi.functions ?? abi).map((func) => ({
            name: func.name,
            index: func.index,
            parameters: func.parameters,
            returnType: func.returnType,
            visibility: func.visibility ?? 'public',
        }));
        return functions.map(func => ({
            name: func.name,
            index: func.index,
            parameters: func.parameters,
            signature: this.generateFunctionSignature(func)
        }));
    }
    /**
     * Generate function signature string
     */
    static generateFunctionSignature(func) {
        const paramStrings = func.parameters.map(param => `${param.name}: ${param.type}${param.optional ? '?' : ''}`);
        const returnType = func.returnType ? ` -> ${func.returnType}` : '';
        return `${func.name}(${paramStrings.join(', ')})${returnType}`;
    }
    /**
     * Validate script ABI structure
     */
    static validateABI(abi) {
        const errors = [];
        if (!abi || typeof abi !== 'object') {
            errors.push('ABI must be an object');
            return { valid: false, errors };
        }
        if (typeof abi.name !== 'string' || abi.name.length === 0) {
            errors.push('ABI must have a non-empty name');
        }
        const functions = normalizeAbiFunctions(abi.functions ?? abi);
        if (functions.length === 0) {
            errors.push('ABI must have at least one function');
        }
        for (let i = 0; i < functions.length; i++) {
            const func = functions[i];
            const funcErrors = this.validateFunction(func, i);
            errors.push(...funcErrors);
        }
        return {
            valid: errors.length === 0,
            errors
        };
    }
    /**
     * Validate function definition
     */
    static validateFunction(func, index) {
        const errors = [];
        const prefix = `Function ${index}`;
        if (typeof func.name !== 'string' || func.name.length === 0) {
            errors.push(`${prefix}: must have a non-empty name`);
        }
        if (typeof func.index !== 'number' || func.index < 0) {
            errors.push(`${prefix}: must have a non-negative index`);
        }
        if (!Array.isArray(func.parameters)) {
            errors.push(`${prefix}: must have a parameters array`);
        }
        if (func.visibility && !['public', 'private'].includes(func.visibility)) {
            errors.push(`${prefix}: visibility must be 'public' or 'private'`);
        }
        return errors;
    }
    // Utility methods for binary data parsing
    static readU32(data, offset) {
        return (data[offset] |
            (data[offset + 1] << 8) |
            (data[offset + 2] << 16) |
            (data[offset + 3] << 24)) >>> 0; // Convert to unsigned
    }
    static readU64(data, offset) {
        // Read as two 32-bit values and combine (JavaScript limitation for large numbers)
        const low = this.readU32(data, offset);
        const high = this.readU32(data, offset + 4);
        return low + (high * 0x100000000);
    }
    static arraysEqual(a, b) {
        if (a.length !== b.length)
            return false;
        for (let i = 0; i < a.length; i++) {
            if (a[i] !== b[i])
                return false;
        }
        return true;
    }
}
/**
 * Client-agnostic metadata cache
 */
export class MetadataCache {
    cache = new Map();
    defaultTTL = 5 * 60 * 1000; // 5 minutes
    /**
     * Get metadata from cache or fetch
     */
    async getMetadata(scriptAddress, fetcher, ttl = this.defaultTTL) {
        const now = Date.now();
        const cached = this.cache.get(scriptAddress);
        if (cached && (now - cached.timestamp) < cached.ttl) {
            return cached.metadata;
        }
        // Fetch fresh metadata
        const metadata = await fetcher(scriptAddress);
        // Cache the result
        this.cache.set(scriptAddress, {
            metadata,
            timestamp: now,
            ttl
        });
        return metadata;
    }
    /**
     * Invalidate cache entry
     */
    invalidate(scriptAddress) {
        this.cache.delete(scriptAddress);
    }
    /**
     * Clear expired entries
     */
    cleanup() {
        const now = Date.now();
        for (const [address, entry] of this.cache.entries()) {
            if ((now - entry.timestamp) >= entry.ttl) {
                this.cache.delete(address);
            }
        }
    }
    /**
     * Get cache statistics
     */
    getStats() {
        const now = Date.now();
        return {
            size: this.cache.size,
            hitRate: 0, // Would need to track hits/misses
            entries: Array.from(this.cache.entries()).map(([address, entry]) => ({
                address,
                age: now - entry.timestamp,
                ttl: entry.ttl
            }))
        };
    }
}
//# sourceMappingURL=index.js.map