export interface NormalizedABIFunction {
  name: string;
  index: number;
  parameters: Array<{
    name: string;
    type: string;
    optional?: boolean;
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
          optional: param.optional ?? false,
          isAccount: param.isAccount ?? param.is_account ?? false,
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
