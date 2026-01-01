// @ts-nocheck
/**
 * Minimal TOML parser supporting the fields used in five.toml.
 * Handles sections, strings, booleans, and numbers.
 * Platform-agnostic: Does not rely on Node.js 'fs'.
 */
export function parseToml(content) {
    const result = {};
    let current = result;
    const lines = content.split(/\r?\n/);
    for (const rawLine of lines) {
        const line = rawLine.split('#')[0].trim();
        if (!line)
            continue;
        const sectionMatch = line.match(/^\[(.+)\]$/);
        if (sectionMatch) {
            const sectionName = sectionMatch[1].trim();
            result[sectionName] = result[sectionName] || {};
            current = result[sectionName];
            continue;
        }
        const eq = line.indexOf('=');
        if (eq === -1)
            continue;
        const key = line.slice(0, eq).trim();
        const rawValue = line.slice(eq + 1).trim();
        current[key] = parseTomlValue(rawValue);
    }
    return result;
}
function parseTomlValue(raw) {
    if ((raw.startsWith('"') && raw.endsWith('"')) || (raw.startsWith("'") && raw.endsWith("'"))) {
        return raw.slice(1, -1);
    }
    if (raw === 'true')
        return true;
    if (raw === 'false')
        return false;
    const num = Number(raw);
    if (!Number.isNaN(num))
        return num;
    return raw;
}
//# sourceMappingURL=toml.js.map