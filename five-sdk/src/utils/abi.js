/**
 * Normalize ABI function definitions that may be emitted as either arrays (FIVEABI)
 * or maps (SimpleABI) into a consistent array format.
 */
export function normalizeAbiFunctions(abiFunctions) {
    if (!abiFunctions)
        return [];
    const functionsArray = Array.isArray(abiFunctions)
        ? abiFunctions
        : Object.entries(abiFunctions).map(([name, func]) => ({
            name,
            ...(func || {}),
        }));
    return functionsArray
        .map((func, idx) => {
        const parameters = Array.isArray(func.parameters) ? func.parameters : [];
        return {
            name: func.name ?? `function_${func.index ?? idx}`,
            index: typeof func.index === 'number' ? func.index : idx,
            parameters: parameters.map((param, paramIdx) => ({
                name: param.name ?? `param${paramIdx}`,
                type: param.type ?? param.param_type ?? param.paramType ?? '',
                optional: param.optional ?? false,
            })),
            returnType: func.returnType ?? func.return_type,
            accounts: func.accounts ?? [],
            visibility: func.visibility ??
                (func.is_public === false ? 'private' : 'public'),
        };
    })
        .sort((a, b) => a.index - b.index);
}
//# sourceMappingURL=abi.js.map