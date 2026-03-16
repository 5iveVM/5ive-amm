/**
 * WASM Loader for Five VM
 * 
 * Loader that uses the main wasm-pack generated entry point.
 * Compatible with both Node.js and Browser environments.
 */

// Store the initialized module instance globally within this module's scope.
let wasmModule: any = null;

import { ConfigManager } from '../config/ConfigManager.js';

export async function getWasmModule(): Promise<any> {
  if (wasmModule) {
    return wasmModule;
  }

  // Build candidate list from config and sensible defaults
  const cfg = await ConfigManager.getInstance().get();
  const prefer = cfg.wasm?.loader || 'auto';
  const configured = Array.isArray(cfg.wasm?.modulePaths) ? cfg.wasm!.modulePaths! : [];

  const nodeCandidates = [
    '../../five_vm_wasm.cjs',
    '../five_vm_wasm.cjs',
    './five_vm_wasm.cjs',
    '../../five_vm_wasm.js',
    '../five_vm_wasm.js',
    './five_vm_wasm.js', // Sibling check
  ];
  const bundlerCandidates = [
    '../../assets/vm/five_vm_wasm.cjs',
    '../assets/vm/five_vm_wasm.cjs',
    '../../assets/vm/five_vm_wasm.js',
    '../assets/vm/five_vm_wasm.js',
    // Fallback for direct import (bundler alias)
    'five-wasm',
    'five-vm-wasm'
  ];

  let candidates: string[] = [];
  candidates.push(...configured);

  // Detect environment to prioritize candidates
  const isNode = typeof process !== 'undefined' && process.versions != null && process.versions.node != null;

  if (prefer === 'node' || (prefer === 'auto' && isNode)) {
    candidates.push(...nodeCandidates);
    candidates.push(...bundlerCandidates);
  } else {
    candidates.push(...bundlerCandidates); // Bundler/Browser preferred
    candidates.push(...nodeCandidates);
  }

  const tried: Array<{ path: string; error: any }> = [];

  for (const candidate of candidates) {
    try {
      const mod = await import(candidate as string);

      // Node.js specific initialization using fs/path (Dynamic Import to avoid Browser errors)
      if (mod && typeof (mod as any).initSync === 'function' && isNode) {
        try {
          const fs = await import('fs');
          const path = await import('path');
          const url = await import('url');

          const dirname = path.dirname;
          const resolve = path.resolve;
          const fileURLToPath = url.fileURLToPath;
          const existsSync = fs.existsSync;
          const readFileSync = fs.readFileSync;

          // Resolve path safely
          let here;
          try {
            here = dirname(fileURLToPath(import.meta.url));
          } catch (e) {
            // Fallback if import.meta.url is not file://
            here = process.cwd();
          }

          const wasmFiles = [
            resolve(here, '../five_vm_wasm_bg.wasm'),
            resolve(here, '../../five_vm_wasm_bg.wasm'),
            resolve(here, '../assets/vm/five_vm_wasm_bg.wasm'),
            resolve(here, '../../assets/vm/five_vm_wasm_bg.wasm'),
          ];
          for (const wf of wasmFiles) {
            if (existsSync(wf)) {
              (mod as any).initSync(readFileSync(wf));
              break;
            }
          }
        } catch (syncErr) {
          // Ignore sync init errors in node, might work with default init
          // tried.push({ path: candidate, error: syncErr });
        }
      }

      // Universal initialization (Browser/Node) if default export is init function
      if (mod && typeof (mod as any).default === 'function') {
        try {
          const initialized = await (mod as any).default();
          const normalizedInit = resolveEncoderModule(initialized);
          if (normalizedInit) {
            wasmModule = normalizedInit;
            return wasmModule;
          }
        } catch (initErr) {
          tried.push({ path: candidate, error: initErr });
        }
      }

      const normalized = resolveEncoderModule(mod);
      if (normalized) {
        wasmModule = normalized;
        return wasmModule;
      }
      tried.push({ path: candidate, error: 'Module missing ParameterEncoder export' });
    } catch (e) {
      tried.push({ path: candidate, error: e });
    }
  }

  const attempted = tried
    .map(t => `  - ${t.path}: ${t.error instanceof Error ? t.error.message : String(t.error)}`)
    .join('\n');

  throw new Error(
    `Five VM WASM module not found or failed to load. Please ensure five-wasm is built.\nAttempts:\n${attempted}`
  );
}
  const resolveEncoderModule = (mod: any): any | null => {
    if (!mod) return null;
    if (mod.ParameterEncoder) return mod;
    if (mod.default && mod.default.ParameterEncoder) return mod.default;
    return null;
  };
