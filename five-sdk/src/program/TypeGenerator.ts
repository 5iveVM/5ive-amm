/**
 * TypeGenerator - Generate TypeScript types from Five VM script ABI
 *
 * Creates type-safe interfaces for:
 * - Function parameters (accounts and data)
 * - Return types
 * - Type-safe builder methods
 *
 * Example generated interface:
 * ```typescript
 * export interface CounterProgram {
 *   initialize(params: {
 *     accounts: { counter: PublicKey | string; owner: PublicKey | string };
 *   }): FunctionBuilder;
 *
 *   add_amount(params: {
 *     accounts: { counter: PublicKey | string; owner: PublicKey | string };
 *     args: { amount: number };
 *   }): FunctionBuilder;
 * }
 * ```
 */

import type { ScriptABI, FunctionDefinition, ParameterDefinition } from '../metadata/index.js';
import { normalizeAbiFunctions } from '../utils/abi.js';

export interface TypeGeneratorOptions {
  /** Name of the generated program interface */
  scriptName?: string;
  /** Enable debug logging */
  debug?: boolean;
  /** Custom namespace for generated types */
  namespace?: string;
  /** Include JSDoc comments */
  includeJSDoc?: boolean;
}

/**
 * TypeGenerator creates TypeScript interfaces from ABI
 */
export class TypeGenerator {
  private abi: ScriptABI;
  private options: TypeGeneratorOptions;

  constructor(abi: ScriptABI, options?: TypeGeneratorOptions) {
    this.abi = {
      ...abi,
      functions: normalizeAbiFunctions((abi as any).functions ?? abi).map((func) => ({
        ...func,
        visibility: func.visibility ?? 'public',
      })) as FunctionDefinition[],
    };
    this.options = {
      scriptName: abi.name || 'Program',
      debug: false,
      includeJSDoc: true,
      ...options,
    };
  }

  /**
   * Generate TypeScript interface definitions from ABI
   *
   * @returns TypeScript code as string
   */
  generate(): string {
    const lines: string[] = [];

    // Header
    lines.push('/**');
    lines.push(` * Auto-generated types for ${this.options.scriptName} program`);
    lines.push(' * Generated from ABI');
    lines.push(' */');
    lines.push('');

    // Generate interface for each function
    const interfaceName = this.generateInterfaceName(this.options.scriptName || 'Program');
    lines.push(`export interface ${interfaceName} {`);

    for (const func of this.abi.functions) {
      // Check both is_public and visibility properties
      const isPublic = func.is_public !== false && func.visibility !== 'private';
      if (isPublic) {
        lines.push(this.generateFunctionSignature(func, 2));
      }
    }

    lines.push('}');
    lines.push('');

    // Generate parameter types for each function
    for (const func of this.abi.functions) {
      const isPublic = func.is_public !== false && func.visibility !== 'private';
      if (isPublic) {
        lines.push(...this.generateFunctionParameterType(func));
      }
    }

    const result = lines.join('\n');

    if (this.options.debug) {
      console.log(`[TypeGenerator] Generated ${this.abi.functions.length} function types`);
    }

    return result;
  }

  /**
   * Generate TypeScript function signature
   *
   * @param func - Function definition
   * @param indent - Indentation level (spaces)
   * @returns Lines of TypeScript code
   */
  private generateFunctionSignature(func: FunctionDefinition, indent: number): string {
    const indentStr = ' '.repeat(indent);
    const paramTypeName = `${this.capitalize(func.name)}Params`;
    let signature = `${indentStr}${func.name}(params: ${paramTypeName}): FunctionBuilder;`;

    if (this.options.includeJSDoc) {
      const lines = [
        `${indentStr}/**`,
        `${indentStr} * Call ${func.name}()`,
        `${indentStr} */`,
        signature,
      ];
      return lines.join('\n');
    }

    return signature;
  }

  /**
   * Generate parameter type definitions for a function
   *
   * @param func - Function definition
   * @returns Lines of TypeScript code
   */
  private generateFunctionParameterType(func: FunctionDefinition): string[] {
    const lines: string[] = [];
    const paramTypeName = `${this.capitalize(func.name)}Params`;

    lines.push(`export interface ${paramTypeName} {`);

    // Extract account and data parameters
    const accountParams = func.parameters.filter((p) => p.is_account === true);
    const dataParams = func.parameters.filter((p) => p.is_account !== true);

    // Generate accounts object if there are account parameters
    if (accountParams.length > 0) {
      lines.push('  accounts: {');
      for (const param of accountParams) {
        lines.push(
          `    ${param.name}: string | { toBase58(): string };`
        );
      }
      lines.push('  };');
    }

    // Generate args object if there are data parameters
    if (dataParams.length > 0) {
      lines.push('  args: {');
      for (const param of dataParams) {
        const typeStr = param.type || param.param_type || 'any';
        const tsType = this.typeToTypeScript(typeStr);
        lines.push(`    ${param.name}: ${tsType};`);
      }
      lines.push('  };');
    }

    lines.push('}');
    lines.push('');

    return lines;
  }

  /**
   * Convert Five VM type to TypeScript type
   *
   * @param fiveType - Five VM type string (e.g., "u64", "Account")
   * @returns TypeScript type string
   */
  private typeToTypeScript(fiveType: string): string {
    const typeMap: Record<string, string> = {
      u8: 'number',
      u16: 'number',
      u32: 'number',
      u64: 'number | bigint',
      u128: 'number | bigint',
      i8: 'number',
      i16: 'number',
      i32: 'number',
      i64: 'number | bigint',
      i128: 'number | bigint',
      f32: 'number',
      f64: 'number',
      bool: 'boolean',
      string: 'string',
      pubkey: 'string | { toBase58(): string }',
      'pubkey[]': 'Array<string | { toBase58(): string }>',
      'u8[]': 'Uint8Array | number[]',
      'u64[]': 'Array<number | bigint>',
    };

    return typeMap[fiveType] || 'any';
  }

  /**
   * Generate interface name from script name
   * E.g., "counter" → "CounterProgram"
   *
   * @param scriptName - Name of the script
   * @returns Interface name
   */
  private generateInterfaceName(scriptName: string): string {
    const base = this.capitalize(scriptName);
    return `${base}Program`;
  }

  /**
   * Capitalize first letter
   *
   * @param str - String to capitalize
   * @returns Capitalized string
   */
  private capitalize(str: string): string {
    return str.charAt(0).toUpperCase() + str.slice(1);
  }

  /**
   * Get the generated ABI as TypeScript AST (for advanced use)
   *
   * @returns Object representation of types
   */
  getABIStructure(): Record<string, any> {
    return {
      programName: this.generateInterfaceName(this.options.scriptName || 'Program'),
      functions: this.abi.functions.map((func) => ({
        name: func.name,
        parameters: func.parameters,
        returnType: func.return_type,
      })),
    };
  }
}
