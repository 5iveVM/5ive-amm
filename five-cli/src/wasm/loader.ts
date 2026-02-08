// WASM loader for Five VM.

let wasmModule: any = null;

import { existsSync, readFileSync } from 'fs';
import { dirname, resolve } from 'path';
import { fileURLToPath } from 'url';
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
    // Prefer top-level dist bundle (node-friendly loader)
    '../../five_vm_wasm.js',
    '../five_vm_wasm.js',
  ];
  const bundlerCandidates = [
    // Then fall back to assets copies (may require bundler, not Node)
    '../../assets/vm/five_vm_wasm.js',
    '../assets/vm/five_vm_wasm.js',
  ];

  let candidates: string[] = [];
  // Prepend any user-configured paths
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
      // eslint-disable-next-line no-await-in-loop
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
      if (mod) {
        wasmModule = mod;
        return wasmModule;
      }
      tried.push({ path: candidate, error: 'Module import returned null/undefined' });
    } catch (e) {
      tried.push({ path: candidate, error: e });
    }
  }

  const attempted = tried
    .map(t => `  - ${t.path}: ${t.error instanceof Error ? t.error.message : String(t.error)}`)
    .join('\n');
  console.error('Fatal Error: Could not load the Five VM WASM module. Tried:\n' + attempted);
  throw new Error(
    `Five VM WASM module not found or failed to load. Please run "npm run build:wasm" to build the required WebAssembly module.`
  );
}
