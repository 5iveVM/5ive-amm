// Compile command.

import { readFile, writeFile, stat, mkdir } from 'fs/promises';
import { join, dirname, extname, basename, isAbsolute, resolve } from 'path';
import { glob } from 'glob';

import { CommandDefinition, CommandContext, ProjectConfig } from '../types.js';
import { FiveSDK, TypeGenerator } from '@5ive-tech/sdk';
import { computeHash, loadProjectConfig, writeBuildManifest, LoadedProjectConfig } from '../project/ProjectLoader.js';
import { success as uiSuccess, error as uiError, section } from '../utils/cli-ui.js';

export const compileCommand: CommandDefinition = {
  name: 'compile',
  description: 'Compile 5ive source to bytecode',
  aliases: ['c'],

  options: [
    {
      flags: '-o, --output <file>',
      description: 'Output file path (default: <input>.five)',
      required: false
    },
    {
      flags: '-t, --target <target>',
      description: 'Compilation target',
      choices: ['vm', 'solana', 'debug', 'test'],
      defaultValue: 'vm'
    },
    {
      flags: '-O, --optimize [level]',
      description: 'Enable optimizations (0-3, default: 2)',
      defaultValue: false
    },
    {
      flags: '--debug',
      description: 'Include debug information in output',
      defaultValue: false
    },
    {
      flags: '--abi <file>',
      description: 'Generate ABI file',
      required: false
    },
    {
      flags: '--analyze',
      description: 'Perform bytecode analysis and show report',
      defaultValue: false
    },
    {
      flags: '--watch',
      description: 'Watch for file changes and recompile',
      defaultValue: false
    },
    {
      flags: '--validate',
      description: 'Validate syntax without compilation',
      defaultValue: false
    },
    {
      flags: '--metrics-output <file>',
      description: 'Write compilation metrics to a file',
      required: false
    },
    {
      flags: '--metrics-format <format>',
      description: 'Metrics export format',
      choices: ['json', 'csv', 'toml'],
      defaultValue: 'json'
    },
    {
      flags: '--error-format <format>',
      description: 'Error output format',
      choices: ['terminal', 'json', 'lsp'],
      defaultValue: 'terminal'
    },
    {
      flags: '--project <path>',
      description: 'Project directory or five.toml path',
      required: false
    },
    {
      flags: '--flat-namespace',
      description: 'Use flat namespace (no module prefixes) for backward compatibility',
      defaultValue: false
    }
  ],

  arguments: [
    {
      name: 'input',
      description: 'Input 5ive source file(s) or glob pattern (optional with five.toml)',
      required: false,
      variadic: true
    }
  ],

  examples: [
    {
      command: '5ive build',
      description: 'Compile project from five.toml entry_point via compiler-owned discovery'
    },
    {
      command: '5ive compile src/main.v',
      description: 'Compile a single source file directly (legacy single-file path)'
    }
  ],

  handler: async (args: any, options: any, context: CommandContext): Promise<void> => {
    const { logger } = context;

    try {
      const projectContext = await loadProjectConfig(options.project, process.cwd());

      if (context.options.verbose) {
        logger.info(
          projectContext
            ? `Using project config at ${projectContext.configPath}`
            : 'No project config found (continuing with explicit file compilation)'
        );
      }

      let inputArgs = args;
      if (Array.isArray(args) && args.length > 0) {
        if (Array.isArray(args[0]) && (typeof args[1] === 'object' && !Array.isArray(args[1]))) {
          inputArgs = args[0];
        }
      }

      const hasExplicitInputs = Array.isArray(inputArgs) && inputArgs.length > 0;

      if (!hasExplicitInputs && projectContext) {
        if (!projectContext.config.entryPoint) {
          throw new Error(`Missing required project.entry_point in ${projectContext.configPath}`);
        }

        await compileProject(projectContext, options, context);
        return;
      }

      const inputFiles = await resolveInputFiles(inputArgs || []);
      if (inputFiles.length === 0) {
        throw new Error(
          'No 5ive source files found. Use `5ive build` from a directory with five.toml, pass --project, or provide explicit file paths.'
        );
      }

      if (options.validate) {
        await validateFiles(inputFiles, context);
      } else if (options.watch) {
        await watchAndCompile(inputFiles, options, context, projectContext ?? undefined);
      } else {
        await compileFiles(inputFiles, options, context, projectContext ?? undefined);
      }
    } catch (error) {
      logger.error('Compilation failed:', error);
      throw error;
    }
  }
};

async function resolveInputFiles(args: string[]): Promise<string[]> {
  const inputFiles: string[] = [];

  for (const arg of args) {
    try {
      const stats = await stat(arg);
      if (stats.isFile() && extname(arg) === '.v') {
        inputFiles.push(arg);
      } else if (stats.isDirectory()) {
        const files = await glob(join(arg, '**/*.v'));
        inputFiles.push(...files);
      }
    } catch {
      const files = await glob(arg);
      inputFiles.push(...files.filter((file) => extname(file) === '.v'));
    }
  }

  return [...new Set(inputFiles)];
}

async function compileProject(
  projectContext: LoadedProjectConfig,
  options: any,
  context: CommandContext
): Promise<void> {
  const { logger } = context;
  const cfg = projectContext.config;

  const outputFile =
    options.output ||
    getDefaultOutputPath(
      cfg.entryPoint || 'main.v',
      options.target || cfg.target || 'vm',
      true,
      cfg,
      projectContext.rootDir
    );

  await mkdir(dirname(outputFile), { recursive: true });

  const compilationOptions: any = {
    optimize: parseOptimizationLevel(options.optimize),
    debug: Boolean(options.debug),
    optimizationLevel: 'production',
    includeMetrics: Boolean(options.metricsOutput),
    metricsFormat: options.metricsFormat || 'json',
    errorFormat: options.errorFormat || 'terminal',
    comprehensiveMetrics: Boolean(options.comprehensiveMetrics),
    metricsOutput: options.metricsOutput,
    target: options.target || cfg.target || 'vm',
    flatNamespace: Boolean(options.flatNamespace)
  };

  const sdkAny = FiveSDK as any;
  const entryPointAbs = resolve(projectContext.rootDir, cfg.entryPoint || '');
  const result: any = typeof sdkAny.compileProject === 'function'
    ? await sdkAny.compileProject(projectContext.configPath, compilationOptions)
    : await sdkAny.compileWithDiscovery(entryPointAbs, compilationOptions);

  if (options.metricsOutput && result.metricsReport?.exported) {
    const metricsPath = isAbsolute(options.metricsOutput)
      ? options.metricsOutput
      : resolve(dirname(outputFile), options.metricsOutput);
    await mkdir(dirname(metricsPath), { recursive: true });
    await writeFile(metricsPath, result.metricsReport.exported);
    if (context.options.verbose) {
      logger.info(`Metrics written to ${metricsPath} (${result.metricsReport.format})`);
    }
  }

  if (!result.success) {
    console.log(uiError('build failed'));
    printCompilationDiagnostics(result);
    const primaryError = extractPrimaryErrorMessage(result);
    throw new Error(primaryError ? `Project build failed: ${primaryError}` : 'Project build failed');
  }

  const sourceText = await readFile(entryPointAbs, 'utf8');
  const bytecodeBytes = result.bytecode instanceof Uint8Array
    ? result.bytecode
    : new Uint8Array(result.bytecode || []);

  if (!result.fiveFile && bytecodeBytes.length > 0) {
    result.fiveFile = {
      bytecode: Buffer.from(bytecodeBytes).toString('base64'),
      abi: result.abi || { functions: {}, fields: [], version: '1.0' },
      version: '1.0',
      metadata: result.metadata || {}
    };
  }

  if (outputFile.endsWith('.five') && result.fiveFile) {
    await writeFile(outputFile, JSON.stringify(result.fiveFile, null, 2));
  } else {
    await writeFile(outputFile, Buffer.from(bytecodeBytes));
  }

  if (options.abi && (result.metadata || result.abi)) {
    await writeFile(options.abi, JSON.stringify(result.metadata || result.abi, null, 2));
    logger.info(`ABI written to ${options.abi}`);
  }

  if (result.metadata || result.abi) {
    try {
      const generator = new TypeGenerator(result.metadata || result.abi);
      const typeDefs = generator.generate();
      const typeFile = outputFile.replace(/(\.five|\.bin|\.fbin)$/, '') + '.d.ts';
      await writeFile(typeFile, typeDefs);
      if (context.options.verbose) logger.info(`Types generated at ${typeFile}`);
    } catch (err) {
      logger.warn(`Failed to generate types: ${err}`);
    }
  }

  const format: 'five' | 'bin' = outputFile.endsWith('.five') ? 'five' : 'bin';
  const artifactBuffer =
    format === 'five'
      ? Buffer.from(JSON.stringify(result.fiveFile ?? {}))
      : Buffer.from(bytecodeBytes);

  const manifest = {
    artifact_path: outputFile,
    abi_path: options.abi,
    compiler_version: result?.metadata?.compilerVersion || 'unknown',
    source_files: [entryPointAbs],
    target: options.target || cfg.target,
    timestamp: new Date().toISOString(),
    hash: artifactBuffer.length ? computeHash(artifactBuffer) : undefined,
    format,
    entry_point: cfg.entryPoint,
    source_dir: cfg.sourceDir
  };

  const manifestPath = await writeBuildManifest(projectContext.rootDir, manifest);
  if (context.options.verbose) {
    logger.info(`Build manifest written to ${manifestPath}`);
  }

  if (options.analyze && bytecodeBytes.length > 0) {
    try {
      const validation = await FiveSDK.validateBytecode(bytecodeBytes, { debug: true });
      displayBytecodeAnalysis({ validation });
    } catch (error) {
      logger.warn(`Failed to analyze bytecode: ${error}`);
    }
  }

  console.log(uiSuccess(`build (${sourceText.length} chars, ${bytecodeBytes.length} bytes)`));
}

async function validateFiles(
  inputFiles: string[],
  _context: CommandContext
): Promise<void> {
  let allValid = true;

  for (const inputFile of inputFiles) {
    try {
      const sourceCode = await readFile(inputFile, 'utf8');
      const result = await FiveSDK.compile(
        {
          filename: inputFile,
          content: sourceCode
        },
        { debug: false }
      );

      if (result.success) {
        console.log(uiSuccess(`${inputFile} - valid syntax`));
      } else {
        console.log(uiError(`${inputFile} - syntax errors`));
        result.errors?.forEach((error) => {
          console.log(`  - ${error.message}`);
        });
        allValid = false;
      }
    } catch (error) {
      console.log(uiError(`${inputFile} - failed to read: ${error}`));
      allValid = false;
    }
  }

  if (!allValid) {
    process.exit(1);
  }
}

async function compileFiles(
  inputFiles: string[],
  options: any,
  context: CommandContext,
  projectContext?: Awaited<ReturnType<typeof loadProjectConfig>>
): Promise<void> {
  const { logger } = context;
  const showPerFile = context.options.verbose || inputFiles.length === 1;

  const results: Array<{ file: string; success: boolean; duration: number }> = [];

  for (const inputFile of inputFiles) {
    const startTime = Date.now();

    try {
      if (context.options.verbose) {
        logger.info(`Compiling ${inputFile}...`);
      }

      const result = await compileSingleFile(inputFile, options, context, projectContext, inputFiles);
      const duration = Date.now() - startTime;
      results.push({ file: inputFile, success: result.success, duration });

      if (result.success) {
        if (showPerFile) {
          console.log(uiSuccess(inputFile));
          console.log(`  ${result.metrics.sourceSize} chars -> ${result.metrics.bytecodeSize} bytes (${duration}ms)`);
        }
      } else {
        console.log(uiError(`${inputFile} failed`));
        printCompilationDiagnostics(result);
      }
    } catch (error) {
      const duration = Date.now() - startTime;
      results.push({ file: inputFile, success: false, duration });
      console.log(uiError(`${inputFile} failed`));
      console.error(`  Error: ${error instanceof Error ? error.message : String(error)}`);
    }
  }

  const successful = results.filter((r) => r.success).length;
  const failed = results.length - successful;
  const totalTime = results.reduce((sum, r) => sum + r.duration, 0);

  if (results.length > 1 || context.options.verbose) {
    console.log('\n' + section('Summary'));
    console.log(`  OK: ${successful}${failed > 0 ? ` | Failed: ${failed}` : ''}`);
    if (context.options.verbose) {
      console.log(`  Total: ${totalTime}ms`);
    }
  }

  if (failed > 0) {
    process.exit(1);
  }
}

async function compileSingleFile(
  inputFile: string,
  options: any,
  context: CommandContext,
  projectContext?: Awaited<ReturnType<typeof loadProjectConfig>>,
  sourceFiles?: string[]
): Promise<any> {
  const { logger } = context;
  const sourceCode = await readFile(inputFile, 'utf8');

  const outputFile =
    options.output ||
    getDefaultOutputPath(
      inputFile,
      options.target || projectContext?.config.target || 'vm',
      true,
      projectContext?.config,
      projectContext?.rootDir
    );

  await mkdir(dirname(outputFile), { recursive: true });

  const compilationOptions: any = {
    optimize: parseOptimizationLevel(options.optimize),
    debug: Boolean(options.debug),
    optimizationLevel: 'production',
    includeMetrics: Boolean(options.metricsOutput),
    metricsFormat: options.metricsFormat || 'json',
    errorFormat: options.errorFormat || 'terminal',
    comprehensiveMetrics: Boolean(options.comprehensiveMetrics),
    metricsOutput: options.metricsOutput,
    flatNamespace: Boolean(options.flatNamespace),
    sourceFile: inputFile
  };

  const { FiveCompilerWasm } = await import('../wasm/compiler.js');
  const wasmCompiler = new FiveCompilerWasm(logger);
  await wasmCompiler.initialize();

  const result: any = await wasmCompiler.compile(sourceCode, compilationOptions);

  if (result.success && !result.fiveFile && result.bytecode) {
    const bytecodeBytes = result.bytecode instanceof Uint8Array
      ? result.bytecode
      : new Uint8Array(result.bytecode);
    result.fiveFile = {
      bytecode: Buffer.from(bytecodeBytes).toString('base64'),
      abi: result.abi || { functions: {}, fields: [], version: '1.0' },
      version: '1.0',
      metadata: result.metadata || {}
    };
  }

  if (options.metricsOutput && result.metricsReport?.exported) {
    const metricsPath = isAbsolute(options.metricsOutput)
      ? options.metricsOutput
      : resolve(dirname(outputFile), options.metricsOutput);
    await mkdir(dirname(metricsPath), { recursive: true });
    await writeFile(metricsPath, result.metricsReport.exported);
    if (context.options.verbose) {
      logger.info(`Metrics written to ${metricsPath} (${result.metricsReport.format})`);
    }
  }

  if (result.success) {
    if (outputFile.endsWith('.five') && result.fiveFile) {
      await writeFile(outputFile, JSON.stringify(result.fiveFile, null, 2));
    } else if (result.bytecode) {
      await writeFile(outputFile, result.bytecode);
    }

    if (options.abi && (result.metadata || result.abi)) {
      await writeFile(options.abi, JSON.stringify(result.metadata || result.abi, null, 2));
      logger.info(`ABI written to ${options.abi}`);
    }

    if (projectContext) {
      const format: 'five' | 'bin' = outputFile.endsWith('.five') ? 'five' : 'bin';
      const artifactBuffer =
        format === 'five'
          ? Buffer.from(JSON.stringify(result.fiveFile ?? {}))
          : Buffer.from(result.bytecode ?? []);
      const manifest = {
        artifact_path: outputFile,
        abi_path: options.abi,
        compiler_version: 'unknown',
        source_files: (sourceFiles || [inputFile]).map((f) =>
          isAbsolute(f) ? f : resolve(projectContext.rootDir, f)
        ),
        target: projectContext.config.target,
        timestamp: new Date().toISOString(),
        hash: artifactBuffer.length ? computeHash(artifactBuffer) : undefined,
        format,
        entry_point: projectContext.config.entryPoint,
        source_dir: projectContext.config.sourceDir
      };
      const manifestPath = await writeBuildManifest(projectContext.rootDir, manifest);
      if (context.options.verbose) {
        logger.info(`Build manifest written to ${manifestPath}`);
      }
    }

    if (result.metadata || result.abi) {
      try {
        const generator = new TypeGenerator(result.metadata || result.abi);
        const typeDefs = generator.generate();
        const typeFile = outputFile.replace(/(\.five|\.bin|\.fbin)$/, '') + '.d.ts';
        await writeFile(typeFile, typeDefs);
        if (context.options.verbose) logger.info(`Types generated at ${typeFile}`);
      } catch (err) {
        logger.warn(`Failed to generate types: ${err}`);
      }
    }
  }

  if (options.analyze && result.success && result.bytecode) {
    try {
      const validation = await FiveSDK.validateBytecode(result.bytecode, { debug: true });
      displayBytecodeAnalysis({ validation });
    } catch (error) {
      logger.warn(`Failed to analyze bytecode: ${error}`);
    }
  }

  return {
    success: result.success,
    errors: result.errors,
    metrics: {
      sourceSize: sourceCode.length,
      bytecodeSize: result.bytecode?.length || 0
    },
    metricsReport: result.metricsReport,
    formattedErrorsTerminal: result.formattedErrorsTerminal
  };
}

async function watchAndCompile(
  inputFiles: string[],
  options: any,
  context: CommandContext,
  projectContext?: Awaited<ReturnType<typeof loadProjectConfig>>
): Promise<void> {
  const { logger } = context;
  const chokidar = await import('chokidar');

  logger.info('Watching for file changes...');
  await compileFiles(inputFiles, { ...options, watch: false }, context, projectContext);

  const watcher = chokidar.watch(inputFiles, {
    persistent: true,
    ignoreInitial: true
  });

  watcher.on('change', async (filePath) => {
    logger.info(`File changed: ${filePath}`);

    try {
      await compileSingleFile(filePath, options, context);
    } catch (error) {
      logger.error(`Recompilation failed: ${error}`);
    }
  });

  process.on('SIGINT', () => {
    logger.info('Stopping file watcher...');
    watcher.close();
    process.exit(0);
  });
}

function getDefaultOutputPath(
  inputFile: string,
  target: string,
  useFiveFormat: boolean = true,
  projectConfig?: ProjectConfig,
  projectRoot?: string
): string {
  const dir = projectConfig
    ? join(projectRoot ?? process.cwd(), projectConfig.buildDir)
    : dirname(inputFile);
  const baseName =
    projectConfig?.outputArtifactName || projectConfig?.name || basename(inputFile, '.v');

  const resolvedTarget = target || projectConfig?.target || 'vm';

  const extensions: Record<string, string> = {
    vm: useFiveFormat ? '.five' : '.bin',
    solana: useFiveFormat ? '.five' : '.so',
    debug: useFiveFormat ? '.five' : '.debug.bin',
    test: useFiveFormat ? '.five' : '.test.bin'
  };

  const ext = extensions[resolvedTarget] || (useFiveFormat ? '.five' : '.bin');
  return join(dir, baseName + ext);
}

function parseOptimizationLevel(optimize: any): boolean {
  if (optimize === false || optimize === 'false' || optimize === '0') {
    return false;
  }
  return true;
}

function displayBytecodeAnalysis(analysis: any): void {
  console.log('\n' + section('Bytecode Analysis'));

  if (analysis.summary) {
    console.log(`  Total size: ${analysis.summary.total_size} bytes`);
    console.log(`  Instructions: ${analysis.summary.total_instructions}`);
    console.log(`  Compute units: ${analysis.summary.total_compute_units}`);
    console.log(`  Functions: ${analysis.summary.has_function_calls ? 'Yes' : 'No'}`);
    console.log(`  Jumps: ${analysis.summary.has_jumps ? 'Yes' : 'No'}`);
  }
}

function printCompilationDiagnostics(result: any): void {
  if (!result) {
    return;
  }

  const diagnostics = Array.isArray(result.errors) ? result.errors : [];
  if (diagnostics.length > 0) {
    for (const diagnostic of diagnostics) {
      const severity = (diagnostic?.severity || 'error').toString().toLowerCase();
      const code = diagnostic?.code ? `[${diagnostic.code}] ` : '';
      const category = diagnostic?.category ? ` (${diagnostic.category})` : '';
      const message = diagnostic?.message || String(diagnostic);
      const header = `${severity}${code}${category}: ${message}`;
      console.error(`  ${header}`);

      const file = diagnostic?.location?.file || diagnostic?.sourceLocation;
      const line = diagnostic?.location?.line ?? diagnostic?.line;
      const column = diagnostic?.location?.column ?? diagnostic?.column;
      if (file || line || column) {
        const atFile = file || 'input.v';
        const atLine = line ?? '?';
        const atColumn = column ?? '?';
        console.error(`    at ${atFile}:${atLine}:${atColumn}`);
      }

      if (typeof diagnostic?.description === 'string' && diagnostic.description.trim().length > 0) {
        console.error(`    note: ${diagnostic.description}`);
      }

      if (typeof diagnostic?.sourceSnippet === 'string' && diagnostic.sourceSnippet.trim().length > 0) {
        const snippet = diagnostic.sourceSnippet.trimEnd().split('\n');
        for (const snippetLine of snippet) {
          console.error(`    ${snippetLine}`);
        }
      } else if (typeof diagnostic?.sourceLine === 'string' && diagnostic.sourceLine.trim().length > 0) {
        console.error(`    source: ${diagnostic.sourceLine.trim()}`);
      }

      const suggestions = collectDiagnosticSuggestions(diagnostic);
      for (const suggestion of suggestions) {
        console.error(`    help: ${suggestion}`);
      }
    }
    return;
  }

  const terminal = typeof result.formattedErrorsTerminal === 'string'
    ? result.formattedErrorsTerminal.trim()
    : '';
  if (terminal) {
    console.log(terminal);
  }
}

function collectDiagnosticSuggestions(diagnostic: any): string[] {
  const suggestions = new Set<string>();

  if (Array.isArray(diagnostic?.suggestions)) {
    for (const entry of diagnostic.suggestions) {
      const text = typeof entry === 'string' ? entry : entry?.message;
      if (typeof text === 'string' && text.trim().length > 0) {
        suggestions.add(text.trim());
      }
    }
  }

  if (typeof diagnostic?.suggestion === 'string' && diagnostic.suggestion.trim().length > 0) {
    suggestions.add(diagnostic.suggestion.trim());
  }

  const code = typeof diagnostic?.code === 'string' ? diagnostic.code : '';
  if (suggestions.size === 0) {
    if (code === 'E2000') {
      suggestions.add('Declare the variable before use with `let <name> = ...`.');
      suggestions.add('Check for spelling differences between parameter/field names and usages.');
    } else if (code === 'E0002') {
      suggestions.add('Check for missing closing `}`, `)`, or an incomplete function signature.');
    } else if (code === 'E0001' || code === 'E0004') {
      suggestions.add('Check for missing punctuation (`;`, `{`, `}`) near the reported statement.');
    }
  }

  return Array.from(suggestions);
}

function extractPrimaryErrorMessage(result: any): string | undefined {
  if (!result) {
    return undefined;
  }

  const firstError = Array.isArray(result.errors) && result.errors.length > 0
    ? result.errors[0]
    : undefined;
  if (firstError) {
    return firstError.message || String(firstError);
  }

  if (typeof result.formattedErrorsTerminal === 'string') {
    const firstLine = result.formattedErrorsTerminal.split('\n').find((line: string) => line.trim().length > 0);
    return firstLine?.trim();
  }

  return undefined;
}
