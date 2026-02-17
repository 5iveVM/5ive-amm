export interface NormalizedABIFunction {
  name: string;
  index: number;
  parameters: Array<{
    name: string;
    type: string;
    param_type?: string;
    optional?: boolean;
    is_account?: boolean;
    isAccount?: boolean;
    attributes?: string[];
  }>;
  returnType?: string;
  accounts?: any[];
  visibility?: 'public' | 'private';
}

/**
 * Normalize ABI function definitions that may be emitted as either arrays (FIVEABI)
 * or maps (SimpleABI) into a consistent array format.
 */
export function normalizeAbiFunctions(abiFunctions: unknown): NormalizedABIFunction[] {
  if (!abiFunctions) return [];

  const functionsArray = Array.isArray(abiFunctions)
    ? abiFunctions
    : Object.entries(abiFunctions as Record<string, any>).map(
        ([name, func]): Record<string, any> => ({
          name,
          ...(func || {}),
        }),
      );

  return functionsArray
    .map((func: any, idx: number) => {
      const parameters = Array.isArray(func.parameters) ? func.parameters : [];

      return {
        name: func.name ?? `function_${func.index ?? idx}`,
        index: typeof func.index === 'number' ? func.index : idx,
        parameters: parameters.map((param: any, paramIdx: number) => ({
          name: param.name ?? `param${paramIdx}`,
          type: param.type ?? param.param_type ?? param.paramType ?? '',
          param_type: param.param_type ?? param.paramType,
          optional: param.optional ?? false,
          is_account: param.is_account ?? param.isAccount ?? false,
          isAccount: param.isAccount ?? param.is_account ?? false,
          attributes: Array.isArray(param.attributes) ? [...param.attributes] : [],
        })),
        returnType: func.returnType ?? func.return_type,
        accounts: func.accounts ?? [],
        visibility:
          func.visibility ??
          (func.is_public === false ? 'private' : 'public'),
      };
    })
    .sort((a, b) => a.index - b.index);
}

/**
 * Find a function in the ABI by name, supporting both flat and qualified names
 *
 * Handles both naming modes:
 * - Flat namespace: "functionName"
 * - Qualified namespace: "module::functionName"
 *
 * Falls back to partial matching if exact match not found.
 */
export function findFunctionInABI(
  abi: any,
  functionName: string,
): NormalizedABIFunction | undefined {
  const functions = normalizeAbiFunctions(abi);

  // Try exact match first
  let func = functions.find(f => f.name === functionName);
  if (func) return func;

  // Try qualified name match (e.g., "module::function" -> "function")
  if (functionName.includes('::')) {
    const parts = functionName.split('::');
    const unqualifiedName = parts[parts.length - 1];
    func = functions.find(f => f.name === unqualifiedName);
    if (func) return func;
  }

  // Try partial match (e.g., "function" matches "module::function")
  func = functions.find(f => f.name.endsWith(`::${functionName}`));
  if (func) return func;

  // No match found
  return undefined;
}

/**
 * Resolve function name to index using ABI
 */
export function resolveFunctionIndex(abi: any, functionName: string): number {
  if (!abi || !abi.functions) {
    throw new Error(
      "No ABI information available for function name resolution",
    );
  }

  // Handle both array format: [{ name: "add", index: 0 }] and object format: { "add": { index: 0 } }
  if (Array.isArray(abi.functions)) {
    // Array format (legacy)
    const func = abi.functions.find((f: any) => f.name === functionName);
    if (!func) {
      const availableFunctions = abi.functions
        .map((f: any) => f.name)
        .join(", ");
      throw new Error(
        `Function '${functionName}' not found in ABI. Available functions: ${availableFunctions}`,
      );
    }
    return func.index;
  } else {
    // Object format (new WASM ABI)
    const func = abi.functions[functionName];
    if (!func) {
      const availableFunctions = Object.keys(abi.functions).join(", ");
      throw new Error(
        `Function '${functionName}' not found in ABI. Available functions: ${availableFunctions}`,
      );
    }
    return func.index;
  }
}
