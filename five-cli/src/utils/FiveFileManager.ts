/**
 * Five File Format Manager
 * 
 * Centralized management for Five file formats (.five, .bin, .v)
 * This is the SINGLE SOURCE OF TRUTH for all Five file operations.
 * 
 * Design Principles:
 * - Single responsibility for file format detection and loading
 * - Consistent error handling across all file operations
 * - Extensible for future file formats
 * - Rich metadata preservation and validation
 * - Performance optimized with caching where appropriate
 */

import { readFile, writeFile, stat } from 'fs/promises';
import { extname, basename } from 'path';
import { Buffer } from 'buffer';

// Types for Five file formats
export interface FiveCompiledFile {
  bytecode: string;  // Base64 encoded bytecode
  abi: {
    functions: Record<string, {
      index: number;
      parameters: any[];
      accounts: any[];
    }>;
    fields: any[];
    version: string;
  };
  version: string;
  disassembly?: string[];  // Human-readable compilation log
  metadata?: {
    sourceFile?: string;
    compilationTime?: number;
    optimizationLevel?: string;
    compilerVersion?: string;
    [key: string]: any;
  };
  debug?: any;
}

export interface LoadedFiveFile {
  bytecode: Uint8Array;
  abi?: any;
  metadata?: any;
  debug?: any;
  format: 'five' | 'bin' | 'v';
  sourceFile: string;
  size: number;
}

export interface FiveFileValidationResult {
  valid: boolean;
  errors: string[];
  warnings: string[];
  format: string;
  size: number;
}

export interface SaveOptions {
  preserveMetadata?: boolean;
  indent?: number;
  compression?: boolean;
}

export interface LoadOptions {
  validateFormat?: boolean;
  requireABI?: boolean;
  cacheable?: boolean;
}

/**
 * Centralized Five File Manager
 * All Five file operations MUST go through this class
 */
export class FiveFileManager {
  private static instance: FiveFileManager;
  private fileCache = new Map<string, LoadedFiveFile>();
  private readonly supportedExtensions = ['.five', '.bin', '.v'];

  /**
   * Singleton pattern to ensure consistent behavior across the CLI
   */
  public static getInstance(): FiveFileManager {
    if (!FiveFileManager.instance) {
      FiveFileManager.instance = new FiveFileManager();
    }
    return FiveFileManager.instance;
  }

  /**
   * Universal file loader - handles ALL Five file formats
   * This is the PRIMARY method that all commands should use
   */
  async loadFile(filePath: string, options: LoadOptions = {}): Promise<LoadedFiveFile> {
    // Input validation
    await this.validateFilePath(filePath);

    const ext = extname(filePath).toLowerCase();
    const cacheKey = `${filePath}:${JSON.stringify(options)}`;

    // Check cache if enabled
    if (options.cacheable && this.fileCache.has(cacheKey)) {
      return this.fileCache.get(cacheKey)!;
    }

    let result: LoadedFiveFile;

    switch (ext) {
      case '.five':
        result = await this.loadFiveFile(filePath, options);
        break;
      case '.bin':
        result = await this.loadBinFile(filePath, options);
        break;
      case '.v':
        result = await this.loadSourceFile(filePath, options);
        break;
      default:
        throw new FiveFileError(
          `Unsupported file format: ${ext}. Supported: ${this.supportedExtensions.join(', ')}`,
          'UNSUPPORTED_FORMAT',
          { filePath, extension: ext }
        );
    }

    // Validate result if requested
    if (options.validateFormat) {
      const validation = this.validateFileContent(result);
      if (!validation.valid) {
        throw new FiveFileError(
          `File validation failed: ${validation.errors.join(', ')}`,
          'VALIDATION_FAILED',
          { filePath, errors: validation.errors }
        );
      }
    }

    // Require ABI if requested
    if (options.requireABI && !result.abi) {
      throw new FiveFileError(
        `ABI required but not found in ${ext} file`,
        'ABI_REQUIRED',
        { filePath, format: result.format }
      );
    }

    // Cache if enabled
    if (options.cacheable) {
      this.fileCache.set(cacheKey, result);
    }

    return result;
  }

  /**
   * Save compiled data to .five format
   * This ensures consistent .five file structure across the CLI
   */
  async saveFiveFile(
    filePath: string,
    bytecode: Uint8Array,
    abi: any,
    metadata?: any,
    disassembly?: string[],
    options: SaveOptions = {}
  ): Promise<void> {
    const fiveFile: FiveCompiledFile = {
      bytecode: Buffer.from(bytecode).toString('base64'),
      abi: {
        functions: abi.functions || {},
        fields: abi.fields || [],
        version: abi.version || '1.0'
      },
      version: '1.0',
      metadata: {
        compilationTime: Date.now(),
        compilerVersion: await this.getCompilerVersion(),
        ...metadata
      }
    };

    // Add disassembly if present
    if (disassembly && disassembly.length > 0) {
      fiveFile.disassembly = disassembly;
    }

    // Add debug info if present
    if (metadata?.debug) {
      fiveFile.debug = metadata.debug;
    }

    const jsonContent = JSON.stringify(fiveFile, null, options.indent || 2);

    try {
      await writeFile(filePath, jsonContent, 'utf8');
    } catch (error) {
      throw new FiveFileError(
        `Failed to save .five file: ${error instanceof Error ? error.message : 'Unknown error'}`,
        'SAVE_FAILED',
        { filePath, error }
      );
    }
  }

  /**
   * Save raw bytecode to .bin format
   */
  async saveBinFile(filePath: string, bytecode: Uint8Array): Promise<void> {
    try {
      await writeFile(filePath, bytecode);
    } catch (error) {
      throw new FiveFileError(
        `Failed to save .bin file: ${error instanceof Error ? error.message : 'Unknown error'}`,
        'SAVE_FAILED',
        { filePath, error }
      );
    }
  }

  /**
   * Detect file format without loading content
   */
  detectFormat(filePath: string): 'five' | 'bin' | 'v' | 'unknown' {
    const ext = extname(filePath).toLowerCase();
    switch (ext) {
      case '.five': return 'five';
      case '.bin': return 'bin';
      case '.v': return 'v';
      default: return 'unknown';
    }
  }

  /**
   * Validate file format and content
   */
  validateFileContent(file: LoadedFiveFile): FiveFileValidationResult {
    const errors: string[] = [];
    const warnings: string[] = [];

    // Basic validation
    if (!file.bytecode || file.bytecode.length === 0) {
      errors.push('Bytecode is empty or missing');
    }

    // Five-specific validation
    if (file.format === 'five') {
      if (!file.abi) {
        errors.push('ABI is missing in .five file');
      } else {
        if (!file.abi.functions) {
          warnings.push('No functions defined in ABI');
        }
        if (!file.abi.version) {
          warnings.push('ABI version not specified');
        }
      }
    }

    // Bytecode validation
    if (file.bytecode.length < 4) {
      errors.push('Bytecode too short to be valid');
    }

    // Check Five VM magic bytes (5IVE)
    const expectedMagic = [0x35, 0x49, 0x56, 0x45]; // "5IVE"
    const actualMagic = Array.from(file.bytecode.slice(0, 4));
    if (!this.arraysEqual(expectedMagic, actualMagic)) {
      warnings.push('Bytecode does not start with Five VM magic bytes');
    }

    return {
      valid: errors.length === 0,
      errors,
      warnings,
      format: file.format,
      size: file.size
    };
  }

  /**
   * Get file information without loading full content
   */
  async getFileInfo(filePath: string): Promise<{
    exists: boolean;
    size: number;
    format: string;
    lastModified: Date;
  }> {
    try {
      const stats = await stat(filePath);
      return {
        exists: true,
        size: stats.size,
        format: this.detectFormat(filePath),
        lastModified: stats.mtime
      };
    } catch (error) {
      return {
        exists: false,
        size: 0,
        format: 'unknown',
        lastModified: new Date(0)
      };
    }
  }

  /**
   * Convert between file formats
   */
  async convertFormat(
    inputPath: string,
    outputPath: string,
    options: { preserveMetadata?: boolean } = {}
  ): Promise<void> {
    const inputFile = await this.loadFile(inputPath, { validateFormat: true });
    const outputFormat = this.detectFormat(outputPath);

    switch (outputFormat) {
      case 'five':
        await this.saveFiveFile(
          outputPath,
          inputFile.bytecode,
          inputFile.abi || { functions: {}, fields: [], version: '1.0' },
          options.preserveMetadata ? inputFile.metadata : undefined
        );
        break;
      case 'bin':
        await this.saveBinFile(outputPath, inputFile.bytecode);
        break;
      default:
        throw new FiveFileError(
          `Cannot convert to format: ${outputFormat}`,
          'CONVERSION_UNSUPPORTED',
          { inputPath, outputPath, outputFormat }
        );
    }
  }

  /**
   * Clear file cache
   */
  clearCache(): void {
    this.fileCache.clear();
  }

  /**
   * Get cache statistics
   */
  getCacheStats(): { size: number; keys: string[] } {
    return {
      size: this.fileCache.size,
      keys: Array.from(this.fileCache.keys())
    };
  }

  // ==================== PRIVATE METHODS ====================

  private async loadFiveFile(filePath: string, options: LoadOptions): Promise<LoadedFiveFile> {
    try {
      const fileContent = await readFile(filePath, 'utf8');
      const stats = await stat(filePath);

      let fiveFile: FiveCompiledFile;
      try {
        fiveFile = JSON.parse(fileContent);
      } catch (parseError) {
        // Check if it's a binary file (starts with FIVE magic bytes)
        // We need to read it as a buffer to check magic bytes
        const buffer = await readFile(filePath);
        const magic = buffer.subarray(0, 4).toString('utf8');

        if (magic === 'FIVE') {
          // It's a binary .five file!
          // For now, we treat it as a binary file since we don't have a full binary parser in TS yet
          return {
            bytecode: new Uint8Array(buffer),
            format: 'bin', // Treat as bin for now, but it's actually binary .five
            sourceFile: filePath,
            size: stats.size,
            metadata: {
              isBinaryFive: true
            }
          };
        }

        throw new FiveFileError(
          'Invalid JSON in .five file and not a valid binary format',
          'JSON_PARSE_ERROR',
          { filePath, parseError }
        );
      }

      // Validate required fields
      if (!fiveFile.bytecode) {
        throw new FiveFileError(
          'Missing bytecode field in .five file',
          'INVALID_FORMAT',
          { filePath }
        );
      }

      if (!fiveFile.abi) {
        throw new FiveFileError(
          'Missing abi field in .five file',
          'INVALID_FORMAT',
          { filePath }
        );
      }

      // Decode base64 bytecode
      let bytecode: Uint8Array;
      try {
        bytecode = new Uint8Array(Buffer.from(fiveFile.bytecode, 'base64'));
      } catch (decodeError) {
        throw new FiveFileError(
          'Invalid base64 bytecode in .five file',
          'BYTECODE_DECODE_ERROR',
          { filePath, decodeError }
        );
      }

      return {
        bytecode,
        abi: fiveFile.abi,
        metadata: fiveFile.metadata,
        debug: fiveFile.debug,
        format: 'five',
        sourceFile: filePath,
        size: stats.size
      };

    } catch (error) {
      if (error instanceof FiveFileError) {
        throw error;
      }
      throw new FiveFileError(
        `Failed to load .five file: ${error instanceof Error ? error.message : 'Unknown error'}`,
        'LOAD_FAILED',
        { filePath, error }
      );
    }
  }

  private async loadBinFile(filePath: string, options: LoadOptions): Promise<LoadedFiveFile> {
    try {
      const bytecode = await readFile(filePath);
      const stats = await stat(filePath);

      return {
        bytecode: new Uint8Array(bytecode),
        format: 'bin',
        sourceFile: filePath,
        size: stats.size
      };

    } catch (error) {
      throw new FiveFileError(
        `Failed to load .bin file: ${error instanceof Error ? error.message : 'Unknown error'}`,
        'LOAD_FAILED',
        { filePath, error }
      );
    }
  }

  private async loadSourceFile(filePath: string, options: LoadOptions): Promise<LoadedFiveFile> {
    try {
      const sourceCode = await readFile(filePath, 'utf8');
      const stats = await stat(filePath);

      // Note: This returns the source code as "bytecode" for now
      // In a real implementation, you might want to compile it first
      const sourceBuffer = Buffer.from(sourceCode, 'utf8');

      return {
        bytecode: new Uint8Array(sourceBuffer),
        format: 'v',
        sourceFile: filePath,
        size: stats.size,
        metadata: {
          isSourceCode: true,
          sourceLength: sourceCode.length
        }
      };

    } catch (error) {
      throw new FiveFileError(
        `Failed to load .v file: ${error instanceof Error ? error.message : 'Unknown error'}`,
        'LOAD_FAILED',
        { filePath, error }
      );
    }
  }

  private async validateFilePath(filePath: string): Promise<void> {
    if (!filePath || typeof filePath !== 'string') {
      throw new FiveFileError(
        'File path is required and must be a string',
        'INVALID_PATH',
        { filePath }
      );
    }

    const info = await this.getFileInfo(filePath);
    if (!info.exists) {
      throw new FiveFileError(
        `File does not exist: ${filePath}`,
        'FILE_NOT_FOUND',
        { filePath }
      );
    }

    const format = this.detectFormat(filePath);
    if (format === 'unknown') {
      throw new FiveFileError(
        `Unsupported file extension. Supported: ${this.supportedExtensions.join(', ')}`,
        'UNSUPPORTED_EXTENSION',
        { filePath, supportedExtensions: this.supportedExtensions }
      );
    }
  }

  private async getCompilerVersion(): Promise<string> {
    // In a real implementation, this would return the actual compiler version
    return '1.0.0';
  }

  private arraysEqual(a: number[], b: number[]): boolean {
    return a.length === b.length && a.every((val, i) => val === b[i]);
  }
}

/**
 * Custom error class for Five file operations
 */
export class FiveFileError extends Error {
  public readonly code: string;
  public readonly details: any;

  constructor(message: string, code: string, details?: any) {
    super(message);
    this.name = 'FiveFileError';
    this.code = code;
    this.details = details;
  }
}

/**
 * Convenience functions for common operations
 * These provide a simple API while still using the centralized manager
 */

/**
 * Quick load function for simple use cases
 */
export async function loadFiveFile(filePath: string): Promise<LoadedFiveFile> {
  const manager = FiveFileManager.getInstance();
  return manager.loadFile(filePath, { validateFormat: true });
}

/**
 * Quick save function for .five files
 */
export async function saveFiveFile(
  filePath: string,
  bytecode: Uint8Array,
  abi: any,
  metadata?: any,
  disassembly?: string[]
): Promise<void> {
  const manager = FiveFileManager.getInstance();
  return manager.saveFiveFile(filePath, bytecode, abi, metadata, disassembly);
}

/**
 * Quick bytecode extraction function
 */
export async function extractBytecode(filePath: string): Promise<Uint8Array> {
  const file = await loadFiveFile(filePath);
  return file.bytecode;
}

/**
 * Quick ABI extraction function
 */
export async function extractABI(filePath: string): Promise<any> {
  const file = await loadFiveFile(filePath);
  if (!file.abi) {
    throw new FiveFileError(
      `No ABI available in ${file.format} file`,
      'NO_ABI',
      { filePath, format: file.format }
    );
  }
  return file.abi;
}

/**
 * Validate any Five file format
 */
export async function validateFiveFile(filePath: string): Promise<FiveFileValidationResult> {
  const manager = FiveFileManager.getInstance();
  const file = await manager.loadFile(filePath);
  return manager.validateFileContent(file);
}