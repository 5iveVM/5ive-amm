#!/usr/bin/env node
import { createHash } from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';

function parseArgs(argv) {
  const map = new Map();
  for (let i = 0; i < argv.length; i += 1) {
    const key = argv[i];
    if (!key.startsWith('--')) {
      continue;
    }
    map.set(key.slice(2), argv[i + 1]);
    i += 1;
  }
  return map;
}

const args = parseArgs(process.argv.slice(2));
const vscodeTarget = args.get('vscode-target');
const rustTarget = args.get('rust-target');
const binaryPath = args.get('binary');
const extensionVersion = process.env.EXT_VERSION || '0.1.0';

if (!vscodeTarget || !rustTarget || !binaryPath) {
  console.error(
    'Usage: node scripts/package-target.mjs --vscode-target <target> --rust-target <target-triple> --binary <path>',
  );
  process.exit(1);
}

if (!fs.existsSync(binaryPath)) {
  console.error(`Binary not found: ${binaryPath}`);
  process.exit(1);
}

const root = process.cwd();
const serverDir = path.join(root, 'server', rustTarget);
const fileName = rustTarget.includes('windows') ? 'five-lsp.exe' : 'five-lsp';
const outputBinary = path.join(serverDir, fileName);
fs.mkdirSync(serverDir, { recursive: true });
fs.copyFileSync(binaryPath, outputBinary);

if (!rustTarget.includes('windows')) {
  fs.chmodSync(outputBinary, 0o755);
}

const hash = createHash('sha256').update(fs.readFileSync(outputBinary)).digest('hex');
const manifestPath = path.join(root, 'server', 'manifest.json');
const manifest = fs.existsSync(manifestPath)
  ? JSON.parse(fs.readFileSync(manifestPath, 'utf8'))
  : { version: extensionVersion, binaries: {} };
manifest.version = extensionVersion;
manifest.binaries[rustTarget] = {
  file: path.relative(path.join(root, 'server'), outputBinary),
  sha256: hash,
};
fs.writeFileSync(manifestPath, JSON.stringify(manifest, null, 2));

const out = `five-language-support-${extensionVersion}-${vscodeTarget}.vsix`;
const packaged = spawnSync(
  process.platform === 'win32' ? 'npx.cmd' : 'npx',
  ['vsce', 'package', '--no-dependencies', '--target', vscodeTarget, '--out', out],
  { stdio: 'inherit', cwd: root },
);
if (packaged.status !== 0) {
  process.exit(packaged.status ?? 1);
}

const checksumPath = `${out}.sha256`;
fs.writeFileSync(checksumPath, `${hash}  ${path.basename(outputBinary)}\n`);
console.log(`Packaged ${out}`);
console.log(`Binary checksum written to ${checksumPath}`);
