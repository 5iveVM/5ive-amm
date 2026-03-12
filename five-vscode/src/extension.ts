import * as fs from 'node:fs';
import * as path from 'node:path';
import * as vscode from 'vscode';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  Trace,
  TransportKind,
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;

function runtimeTargetTriple(): string {
  const key = `${process.platform}-${process.arch}`;
  switch (key) {
    case 'darwin-arm64':
      return 'aarch64-apple-darwin';
    case 'darwin-x64':
      return 'x86_64-apple-darwin';
    case 'linux-arm64':
      return 'aarch64-unknown-linux-gnu';
    case 'linux-x64':
      return 'x86_64-unknown-linux-gnu';
    case 'win32-arm64':
      return 'aarch64-pc-windows-msvc';
    case 'win32-x64':
      return 'x86_64-pc-windows-msvc';
    default:
      throw new Error(`Unsupported runtime platform/arch: ${key}`);
  }
}

function traceFromConfig(value: string): Trace {
  switch (value) {
    case 'messages':
      return Trace.Messages;
    case 'verbose':
      return Trace.Verbose;
    default:
      return Trace.Off;
  }
}

function resolveBinaryPath(context: vscode.ExtensionContext): string {
  const config = vscode.workspace.getConfiguration('five.languageServer');
  const override = config.get<string>('path', '').trim();
  if (override.length > 0) {
    if (!path.isAbsolute(override)) {
      throw new Error('five.languageServer.path must be an absolute path');
    }
    return override;
  }

  const triple = runtimeTargetTriple();
  const binary = process.platform === 'win32' ? 'five-lsp.exe' : 'five-lsp';
  const resolved = context.asAbsolutePath(path.join('server', triple, binary));
  return resolved;
}

async function startLanguageServer(context: vscode.ExtensionContext): Promise<void> {
  const binaryPath = resolveBinaryPath(context);
  if (!fs.existsSync(binaryPath)) {
    throw new Error(
      `five-lsp binary not found at ${binaryPath}. Install a platform VSIX with bundled binaries.`,
    );
  }

  if (process.platform !== 'win32') {
    fs.chmodSync(binaryPath, 0o755);
  }

  const serverOptions: ServerOptions = {
    run: {
      command: binaryPath,
      transport: TransportKind.stdio,
      options: { env: process.env },
    },
    debug: {
      command: binaryPath,
      transport: TransportKind.stdio,
      options: { env: { ...process.env, RUST_LOG: process.env.RUST_LOG ?? 'debug' } },
    },
  };

  const traceConfig = vscode.workspace
    .getConfiguration('five.languageServer')
    .get<string>('trace', 'off');

  const clientOptions: LanguageClientOptions = {
    documentSelector: [
      { scheme: 'file', language: 'five' },
      { scheme: 'untitled', language: 'five' },
    ],
    synchronize: {
      fileEvents: vscode.workspace.createFileSystemWatcher('**/*.{v,five}'),
    },
  };

  client = new LanguageClient('fiveLanguageServer', '5ive Language Server', serverOptions, clientOptions);
  client.setTrace(traceFromConfig(traceConfig));
  await client.start();
}

async function restartLanguageServer(context: vscode.ExtensionContext): Promise<void> {
  if (client) {
    await client.stop();
    client = undefined;
  }
  await startLanguageServer(context);
  await vscode.window.showInformationMessage('5ive language server restarted');
}

async function showLanguageServerStatus(context: vscode.ExtensionContext): Promise<void> {
  const binaryPath = resolveBinaryPath(context);
  const status = client ? 'running' : 'stopped';
  await vscode.window.showInformationMessage(`5ive language server is ${status} (${binaryPath})`);
}

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  context.subscriptions.push(
    vscode.commands.registerCommand('five.restartLanguageServer', () => restartLanguageServer(context)),
    vscode.commands.registerCommand('five.showLanguageServerStatus', () => showLanguageServerStatus(context)),
    vscode.workspace.onDidChangeConfiguration(async (event) => {
      if (event.affectsConfiguration('five.languageServer')) {
        await restartLanguageServer(context);
      }
    }),
  );

  await startLanguageServer(context);
}

export async function deactivate(): Promise<void> {
  if (client) {
    await client.stop();
    client = undefined;
  }
}
