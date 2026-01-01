/**
 * VLE (Variable Length Encoding) Encoder for Five VM Protocol
 * 
 * Uses the existing WASM VLE and Parameter encoders for protocol-compliant
 * encoding of execute instructions used by the frontend implementation.
 * 
 * Execute instruction format:
 * [discriminator(2), vle_function_index, vle_param_count, ...vle_parameters]
 */

// Dynamic WASM module loading
import { existsSync, readFileSync } from 'fs';
import { dirname, resolve } from 'path';
import { fileURLToPath } from 'url';
import { ConfigManager } from '../config/ConfigManager.js';
let wasmModule: any = null;

export interface ParameterDefinition {
  name: string;
  type: string;
}

export interface ParameterValue {
  [key: string]: any;
}

/**
 * Type mapping for VLE encoding - matches WASM module types
 */
// Type mapping for VLE encoding - matches Five protocol types from five-protocol/src/types.rs
const TYPE_IDS: Record<string, number> = {
  'u8': 1,
  'u16': 2,
  'u32': 3,
  'u64': 4,
  'i8': 5,
  'i16': 6,
  'i32': 7,
  'i64': 8,
  'bool': 9,
  'pubkey': 10,
  'string': 11,    // ✅ Now supported in WASM
  'account': 12,   // Account reference type
  'bytes': 12,     // Map to account type for binary data
  'array': 13      // ✅ Now supported in WASM
};

/**
 * VLE Encoder utility class that uses WASM module for protocol compliance
 */
export class VLEEncoder {
  /**
   * Get type ID for VLE encoding
   */
  static getTypeId(type: string): number {
    const typeId = TYPE_IDS[type.toLowerCase()];
    if (typeId === undefined) {
      throw new Error(`Unknown type for VLE encoding: ${type}`);
    }
    return typeId;
  }

  /**
   * Encode execute instruction data using WASM ParameterEncoder
   * This matches the frontend implementation that successfully executes complex functions
   */
  static async encodeExecuteVLE(
    functionIndex: number,
    parameters: ParameterDefinition[] = [],
    values: ParameterValue = {}
  ): Promise<Buffer> {
    if (typeof process !== 'undefined' && process?.env?.NODE_ENV !== 'production') {
      console.log('[VLEEncoder] Encoding execute instruction:', {
        functionIndex,
        parameterCount: parameters.length,
        parameters: parameters.map(p => ({ name: p.name, type: p.type })),
        values
      });
    }

    try {
      // Load WASM module using secure path resolution
      if (!wasmModule) {
        await this.loadWasmModuleSecurely();
      }
      
      // Use PURE VLE compression - encode only values without type information
      const simpleValues = parameters.map(param => {
        const value = values[param.name];
        if (value === undefined || value === null) {
          throw new Error(`Missing value for parameter: ${param.name}`);
        }
        return value;
      });

      console.log('[VLEEncoder] Using PURE VLE compression:', {
        functionIndex,
        simpleValues,
        valueCount: simpleValues.length
      });

      // Call WASM encoder directly with pure values for maximum compression
      const encoded = wasmModule.ParameterEncoder.encode_execute_vle(functionIndex, simpleValues);
      
      // FIXED: WASM ParameterEncoder returns raw VLE data WITHOUT discriminator
      // The discriminator is added by the SDK layer in encodeExecuteInstruction()
      // NO slice(1) needed - that was corrupting the VLE data by removing function index
      const result = Buffer.from(encoded);

      console.log('[VLEEncoder] WASM encoder output (raw VLE data):', {
        bytes: Array.from(encoded),
        totalBytes: result.length,
        hex: result.toString('hex')
      });

      return result;

    } catch (error) {
      // In production mode, we should not fallback to manual encoding
      // The WASM integration should be reliable
      throw new Error(`VLE encoding failed: ${error instanceof Error ? error.message : 'Unknown WASM error'}. Ensure Five VM WASM modules are properly built and available.`);
    }
  }



  /**
   * Parse parameter definitions from JSON string
   */
  static parseParameters(parametersJson: string): { definitions: ParameterDefinition[], values: ParameterValue } {
    try {
      const parsed = JSON.parse(parametersJson);
      
      if (Array.isArray(parsed)) {
        // Array format: [{ name: "a", type: "u64", value: 123 }, ...]
        const definitions: ParameterDefinition[] = [];
        const values: ParameterValue = {};
        
        for (const param of parsed) {
          if (!param.name || !param.type || param.value === undefined) {
            throw new Error('Parameter must have name, type, and value fields');
          }
          definitions.push({ name: param.name, type: param.type });
          values[param.name] = param.value;
        }
        
        return { definitions, values };
      } else {
        // Object format: { "a": 123, "b": 456 } - types must be inferred or provided separately
        throw new Error('Object format parameters require type definitions. Use array format: [{ name: "a", type: "u64", value: 123 }]');
      }
    } catch (error) {
      throw new Error(`Invalid parameters JSON: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  /**
   * Load WASM module explicitly - NO SILENT FALLBACKS (Engineering Integrity Rule #9)
   */
  static async loadWasmModuleSecurely(): Promise<void> {
    // ENGINEERING INTEGRITY: Single canonical WASM module location - no path guessing
    // Robust multi-path resolution relative to this module (ESM):
    // - dist/lib -> dist/assets (../assets)
    // - dist/lib -> package root assets (../../assets)
    // - src/lib  -> src/assets (../assets)
    const cfg = await ConfigManager.getInstance().get();
    const prefer = cfg.wasm?.loader || 'auto';
    const configured = Array.isArray(cfg.wasm?.modulePaths) ? cfg.wasm!.modulePaths! : [];
    const nodeCandidates = [
      '../five_vm_wasm.js',
      '../../five_vm_wasm.js',
    ];
    const bundlerCandidates = [
      '../assets/vm/five_vm_wasm.js',
      '../../assets/vm/five_vm_wasm.js',
      './assets/vm/five_vm_wasm.js',
    ];
    const candidates: string[] = [];
    candidates.push(...configured);
    if (prefer === 'node') {
      candidates.push(...nodeCandidates);
    } else if (prefer === 'bundler') {
      candidates.push(...bundlerCandidates);
    } else {
      candidates.push(...nodeCandidates, ...bundlerCandidates);
    }

    const tried: Array<{ path: string; error: any }> = [];

    for (const candidate of candidates) {
      try {
        console.log(`[VLEEncoder] Attempting to load WASM module: ${candidate}`);
        // Dynamic import resolved relative to this module
        const mod = await import(candidate as string);
        // If initSync is available, prefer initializing with local file bytes to avoid fetch/file URL issues
        if (mod && typeof (mod as any).initSync === 'function') {
          try {
            const here = dirname(fileURLToPath(import.meta.url));
            const wasmFiles = [
              resolve(here, '../five_vm_wasm_bg.wasm'),
              resolve(here, '../../five_vm_wasm_bg.wasm'),
              resolve(here, '../assets/vm/five_vm_wasm_bg.wasm'),
              resolve(here, '../../assets/vm/five_vm_wasm_bg.wasm'),
            ];
            for (const wf of wasmFiles) {
              if (existsSync(wf)) {
                // eslint-disable-next-line no-await-in-loop
                (mod as any).initSync(readFileSync(wf));
                break;
              }
            }
          } catch (syncErr) {
            tried.push({ path: candidate, error: syncErr });
          }
        }
        // Initialize node-friendly wasm-pack bundle if it exposes a default init (fallback)
        if (mod && typeof (mod as any).default === 'function') {
          try {
            // eslint-disable-next-line no-await-in-loop
            await (mod as any).default();
          } catch (initErr) {
            tried.push({ path: candidate, error: initErr });
          }
        }
        if (!mod) throw new Error('Module import returned null/undefined');

        // Validate interface
        if (!mod.ParameterEncoder || !mod.ParameterEncoder.encode_execute_vle) {
          throw new Error('Missing ParameterEncoder.encode_execute_vle');
        }

        wasmModule = mod;
        console.log(`[VLEEncoder] Loaded WASM module successfully from: ${candidate}`);
        return;
      } catch (e) {
        tried.push({ path: candidate, error: e });
      }
    }

    // If we reach here, all attempts failed
    const attempted = tried
      .map(t => `  - ${t.path}: ${t.error instanceof Error ? t.error.message : String(t.error)}`)
      .join('\n');

    const errorMessage = `
CRITICAL: Five VM WASM Module Loading Failed

Attempted Paths:\n${attempted}

REQUIRED ACTION:
1. Ensure Five VM WASM module is built: npm run build:wasm
2. Verify WASM files exist in /assets/vm/ directory
3. Check working directory matches Five CLI root
4. Validate WASM module exports ParameterEncoder interface

This is a hard requirement - Five VM cannot function without WASM module.
`.trim();
      
    throw new Error(errorMessage);
  }

  /**
   * Validate WASM module availability
   */
  static async validateWasmModule(): Promise<{ available: boolean; error?: string }> {
    try {
      if (!wasmModule) {
        await this.loadWasmModuleSecurely();
      }
      
      // Test if ParameterEncoder is available
      if (!wasmModule.ParameterEncoder) {
        return {
          available: false,
          error: 'ParameterEncoder not found in WASM module'
        };
      }
      
      return { available: true };
    } catch (error) {
      return {
        available: false,
        error: `WASM module not available: ${error instanceof Error ? error.message : 'Unknown error'}`
      };
    }
  }

  /**
   * Create typed parameter definitions with validation
   */
  static createTypedParameters(params: Array<{
    name: string;
    type: string;
    value: any;
    optional?: boolean;
  }>): { definitions: ParameterDefinition[], values: ParameterValue } {
    const definitions: ParameterDefinition[] = [];
    const values: ParameterValue = {};
    
    for (const param of params) {
      // Validate type
      if (!TYPE_IDS[param.type.toLowerCase()]) {
        throw new Error(`Unknown parameter type: ${param.type}`);
      }
      
      // Validate required values
      if (!param.optional && (param.value === undefined || param.value === null)) {
        throw new Error(`Missing value for required parameter: ${param.name}`);
      }
      
      definitions.push({ name: param.name, type: param.type });
      if (param.value !== undefined) {
        values[param.name] = param.value;
      }
    }
    
    return { definitions, values };
  }

}
