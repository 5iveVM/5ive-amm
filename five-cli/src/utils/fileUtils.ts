// Legacy file utilities for FiveFileManager compatibility.

import { FiveFileManager, LoadedFiveFile, FiveFileError } from './FiveFileManager.js';

/**
 * @deprecated Use FiveFileManager.loadFile() instead
 */
export async function loadAnyFiveFile(filePath: string): Promise<{
  bytecode: Uint8Array;
  abi?: any;
  format: string;
}> {
  const manager = FiveFileManager.getInstance();
  const file = await manager.loadFile(filePath, { validateFormat: true });
  
  return {
    bytecode: file.bytecode,
    abi: file.abi,
    format: file.format
  };
}

/**
 * @deprecated Use FiveFileManager.extractBytecode() instead
 */
export async function getBytecodeFromFile(filePath: string): Promise<Uint8Array> {
  const { bytecode } = await loadAnyFiveFile(filePath);
  return bytecode;
}

/**
 * @deprecated Use FiveFileManager.extractABI() instead  
 */
export async function getABIFromFile(filePath: string): Promise<any | null> {
  const { abi } = await loadAnyFiveFile(filePath);
  return abi || null;
}

/**
 * @deprecated Use FiveFileManager.detectFormat() instead
 */
export function detectFileFormat(filePath: string): 'five' | 'bin' | 'v' | 'unknown' {
  const manager = FiveFileManager.getInstance();
  return manager.detectFormat(filePath);
}

/**
 * @deprecated Use FiveFileManager.validateFileContent() instead
 */
export async function validateBytecodeFile(filePath: string): Promise<boolean> {
  try {
    const manager = FiveFileManager.getInstance();
    const file = await manager.loadFile(filePath);
    const validation = manager.validateFileContent(file);
    return validation.valid;
  } catch (error) {
    return false;
  }
}
