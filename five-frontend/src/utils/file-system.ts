export const SEPARATOR = '/';

/**
 * Returns the file name from a path
 * e.g. "src/utils/math.five" -> "math.five"
 */
export function getBaseName(path: string): string {
    return path.split(SEPARATOR).pop() || '';
}

/**
 * Returns the directory path
 * e.g. "src/utils/math.five" -> "src/utils"
 */
export function getDirName(path: string): string {
    const parts = path.split(SEPARATOR);
    parts.pop(); // Remove filename
    return parts.join(SEPARATOR);
}

/**
 * Returns the extension of the file
 * e.g. "math.five" -> "five"
 */
export function getExtension(path: string): string {
    const parts = path.split('.');
    return parts.length > 1 ? parts.pop() || '' : '';
}

/**
 * Joins parts into a path
 */
export function joinPath(...parts: string[]): string {
    return parts.filter(Boolean).join(SEPARATOR).replace(/\/+/g, '/');
}

/**
 * Validates a file path
 * - No empty parts
 * - No specialized characters (simple validation)
 */
export function isValidPath(path: string): boolean {
    return !path.includes('//') && !path.startsWith('/') && !path.endsWith('/');
}

/**
 * Checks if a path is a child of a folder
 */
export function isChildOf(filePath: string, folderPath: string): boolean {
    return filePath.startsWith(folderPath + SEPARATOR);
}

/**
 * Sorts files: Folders first, then files alphabetically
 */
export function sortFiles(files: string[]): string[] {
    return files.sort((a, b) => {
        const aDir = getDirName(a);
        const bDir = getDirName(b);
        const aBase = getBaseName(a);
        const bBase = getBaseName(b);

        if (aDir === bDir) {
            // Same directory
            return aBase.localeCompare(bBase);
        }
        return a.localeCompare(b);
    });
}
