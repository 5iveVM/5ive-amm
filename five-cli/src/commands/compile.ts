// Compile command.

import { readFile, writeFile, stat, mkdir } from 'fs/promises';
import { join, dirname, extname, basename, isAbsolute, resolve } from 'path';
import { glob } from 'glob';
import ora from 'ora';

import { CommandDefinition, CommandContext, CLIOptions, ProjectConfig } from '../types.js';
import { CompilationOptions, FiveSDK, TypeGenerator } from '@5ive-tech/sdk';
import { computeHash, loadProjectConfig, writeBuildManifest } from '../project/ProjectLoader.js';
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
      command: '5ive compile src/main.v',
      description: 'Compile a single 5ive source file'
    },
    {
      command: '5ive compile src/**/*.v -o build/',
      description: 'Compile all 5ive files in src directory'
    },
    {
      command: '5ive compile src/main.v -t solana -O 3 --abi main.abi.json',
      description: 'Compile for Solana with maximum optimization and ABI generation'
    },
    {
      command: '5ive compile src/main.v --analyze --debug',
      description: 'Compile with debug info and bytecode analysis'
    },
    {
      command: '5ive compile src/**/*.v --watch',
      description: 'Watch for changes and auto-recompile'
    }
  ],

  handler: async (args: any, options: any, context: CommandContext): Promise<void> => {
    const { logger } = context;

    try {
      // Load project configuration if available
      const projectContext = await loadProjectConfig(options.project, process.cwd());
      if (context.options.verbose) {
        logger.info(
          projectContext
            ? `Using project config at ${projectContext.configPath}`
            : 'No project config found (continuing with CLI arguments)'
        );
      }

      // Initialize SDK silently

      // Resolve input files - args might be [inputs, options] or just inputs depending on how CLI framework calls it
      let inputArgs = args;
      if (Array.isArray(args) && args.length > 0) {
        // If first element is an array (the actual inputs) and second is an object (options),
        // then args was passed as [inputs, options]
        if (Array.isArray(args[0]) && (typeof args[1] === 'object' && !Array.isArray(args[1]))) {
          inputArgs = args[0];
        }
      }

      // Add auto-discover flag to options for resolution
      const resolveOptions = {
        ...options,
        autoDiscover: options.autoDiscover // Assuming this is passed from CLI args
      };

      const { files: inputFiles, mode } = await resolveInputFiles(inputArgs, resolveOptions, logger, projectContext);

      if (inputFiles.length === 0) {
        throw new Error(
          'No 5ive source files found. Provide input paths, or run from a project with five.toml (entry_point/source_dir), or pass --project <path>.'
        );
      }

      // Only show file count in verbose mode
      if (context.options.verbose) {
        logger.info(`Found ${inputFiles.length} source file(s) to compile (mode: ${mode})`);
      }

      // Handle different modes
      if (options.validate) {
        await validateFiles(inputFiles, context);
      } else if (options.watch) {
        await watchAndCompile(inputFiles, options, context, projectContext);
      } else {
        // If multi-file mode is detected, use compileMultiProject
        if (mode === 'multi-auto' || mode === 'multi-explicit' || projectContext?.config.multiFileMode) {
          // For multi-file compilation, pass all files including dependencies
          // If auto-discover was used, inputFiles already contains all discovered modules
          if (projectContext) {
            await compileMultiProject(inputFiles, options, context, projectContext);
          }
        } else {
          await compileFiles(inputFiles, options, context, projectContext);
        }
      }

    } catch (error) {
      logger.error('Compilation failed:', error);
      throw error;
    }
  }
};

/**
 * Resolve input files from arguments and glob patterns
 */
async function resolveInputFiles(
  args: string[],
  options: any,
  logger: any,
  projectContext?: Awaited<ReturnType<typeof loadProjectConfig>>
): Promise<{ files: string[], mode: 'single' | 'multi-auto' | 'multi-explicit' }> {
  const inputFiles: string[] = [];

  // If no args and project config exists, derive from config
  if ((!args || args.length === 0) && projectContext) {
    const root = projectContext.rootDir;
    const cfg = projectContext.config;

    // If modules are defined in config, use them
    if (cfg.modules && Object.keys(cfg.modules).length > 0) {
      const moduleFiles: string[] = [];
      const entryPoint = cfg.entryPoint ? join(root, cfg.entryPoint) : null;

      if (entryPoint) {
        moduleFiles.push(entryPoint);
      }

      for (const [_, files] of Object.entries(cfg.modules)) {
        for (const file of files) {
          const absPath = isAbsolute(file) ? file : join(root, file);
          if (absPath !== entryPoint) {
            moduleFiles.push(absPath);
          }
        }
      }

      // If we have an entry point or modules, return valid configuration
      if (moduleFiles.length > 0) {
        // Ensure unique files
        const uniqueFiles = [...new Set(moduleFiles)];

        if (logger && logger.debug) {
          logger.debug(`Resolved ${uniqueFiles.length} files from five.toml [modules] section`);
        }

        return { files: uniqueFiles, mode: 'multi-explicit' };
      }
    }

    const multiFileMode: string = typeof cfg.multiFileMode === 'string' ? cfg.multiFileMode : 'disabled';

    if (multiFileMode !== 'disabled' && !cfg.entryPoint) {
      throw new Error('multi_file_mode is enabled but entry_point is not set in five.toml');
    }

    if (cfg.entryPoint) {
      const entryPoint = join(root, cfg.entryPoint);

      // Auto discovery mode from config
      if ((multiFileMode as string) === 'auto') {
        // Dynamically import to avoid circular dependency issues
        const { FiveCompilerWasm } = await import('../wasm/compiler.js');
        const compiler = new FiveCompilerWasm(logger);
        await compiler.initialize();
        const discovered = await compiler.discoverModules(entryPoint);
        return { files: [entryPoint, ...discovered], mode: 'multi-auto' };
      }

      inputFiles.push(entryPoint);

      // NEW: Auto-detect multi-file mode like five-frontend-2
      // Scan for all .v files in sourceDir and use multi-file if multiple files found
      const pattern = join(root, cfg.sourceDir || 'src', '**/*.v');
      const allVFiles = await glob(pattern);
      const uniqueVFiles = [...new Set(allVFiles)];

      // If multiple .v files exist, use multi-file compilation
      // But only if we didn't use modules section (implicit vs explicit)
      if (uniqueVFiles.length > 1) {
        if (logger && logger.debug) {
          logger.debug(`Detected ${uniqueVFiles.length} .v files - using multi-file compilation mode`);
        }
        // Return all files with entry point first
        const filesWithEntry = [entryPoint, ...uniqueVFiles.filter(f => f !== entryPoint)];
        return { files: [...new Set(filesWithEntry)], mode: 'multi-explicit' };
      }

      return { files: inputFiles, mode: 'single' };
    }

    // default: compile all .v files in sourceDir
    const pattern = join(root, cfg.sourceDir, '**/*.v');
    const files = await glob(pattern);
    inputFiles.push(...files);

    // If multiple files found, use multi-file mode
    const uniqueFiles = [...new Set(inputFiles)];
    const mode = uniqueFiles.length > 1 ? 'multi-explicit' : 'single';

    return { files: uniqueFiles, mode };
  }

  // Check for CLI flags
  if (options.autoDiscover) {
    // Auto-discover mode requires a single entry point
    if (args.length !== 1) {
      throw new Error('--auto-discover requires exactly one entry point file');
    }

    const entryPoint = args[0];
    // Dynamically import to avoid circular dependency issues
    const { FiveCompilerWasm } = await import('../wasm/compiler.js');
    const compiler = new FiveCompilerWasm(logger);
    await compiler.initialize();
    const discovered = await compiler.discoverModules(entryPoint);

    // Return entry point + discovered modules
    return { files: [entryPoint, ...discovered], mode: 'multi-auto' };
  }

  for (const arg of args) {
    try {
      const stats = await stat(arg);

      if (stats.isFile() && extname(arg) === '.v') {
        inputFiles.push(arg);
      } else if (stats.isDirectory()) {
        // Search for .v files in directory
        const files = await glob(join(arg, '**/*.v'));
        inputFiles.push(...files);
      }
    } catch {
      // Try as glob pattern
      const files = await glob(arg);
      const fiveFiles = files.filter(file => extname(file) === '.v');
      inputFiles.push(...fiveFiles);
    }
  }

  const uniqueFiles = [...new Set(inputFiles)];

  // If multiple files explicitly provided, treat as multi-explicit
  if (uniqueFiles.length > 1) {
    return { files: uniqueFiles, mode: 'multi-explicit' };
  }

  return { files: uniqueFiles, mode: 'single' };
}

/**
 * Validate files without compilation using 5IVE SDK
 */
async function validateFiles(
  inputFiles: string[],
  context: CommandContext
): Promise<void> {
  const { logger } = context;
  let allValid = true;

  for (const inputFile of inputFiles) {
    try {
      const sourceCode = await readFile(inputFile, 'utf8');

      // Use 5IVE SDK to validate bytecode
      const result = await FiveSDK.compile(sourceCode, { debug: false });

      if (result.success) {
        console.log(uiSuccess(`${inputFile} - valid syntax`));
      } else {
        console.log(uiError(`${inputFile} - syntax errors`));
        if ((result as any).formattedErrorsTerminal) {
          console.log((result as any).formattedErrorsTerminal);
        } else {
          result.errors?.forEach(error => {
            console.log(`  - ${error.message}`);
          });
        }
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

/**
 * Compile multiple files
 */
async function compileFiles(
  inputFiles: string[],
  options: any,
  context: CommandContext,
  projectContext?: Awaited<ReturnType<typeof loadProjectConfig>>
): Promise<void> {
  const { logger } = context;
  const showPerFile = context.options.verbose || inputFiles.length === 1;

  if (projectContext?.config.multiFileMode) {
    await compileMultiProject(inputFiles, options, context, projectContext);
    return;
  }

  const results: Array<{ file: string; success: boolean; duration: number }> = [];

  for (const inputFile of inputFiles) {
    const startTime = Date.now();

    try {
      if (context.options.verbose) {
        logger.info(`Compiling ${inputFile}...`);
      }

      const result = await compileSingleFile(
        inputFile,
        options,
        context,
        projectContext,
        inputFiles
      );

      const duration = Date.now() - startTime;
      results.push({ file: inputFile, success: result.success, duration });

      if (result.success) {
        if (showPerFile) {
          console.log(uiSuccess(inputFile));
          console.log(`  ${result.metrics.sourceSize} chars -> ${result.metrics.bytecodeSize} bytes (${duration}ms)`);
        }
      } else {
        console.log(uiError(`${inputFile} failed`));

        // Display compilation errors (enhanced error system)
        if (context.options.verbose) {
          console.log("DEBUG: formattedErrorsTerminal prop:", (result as any).formattedErrorsTerminal);
        }
        if ((result as any).formattedErrorsTerminal) {
          console.log((result as any).formattedErrorsTerminal);
        } else if (result.errors) {
          for (const error of result.errors) {
            // Check if this is an enhanced error with rich information
            if ((error as any).code && (error as any).severity) {
              const enhancedError = error as any;

              // Display the error with code and category
              console.error(`  Error ${enhancedError.code}: ${enhancedError.message}`);

              // Display description if available
              if (enhancedError.description) {
                console.error(`    ${enhancedError.description}`);
              }

              // Display source location if available
              if (enhancedError.location) {
                const loc = enhancedError.location;
                console.error(`    at line ${loc.line}, column ${loc.column}${loc.file ? ` in ${loc.file}` : ''}`);
              }

              // Display suggestions if available
              if (enhancedError.suggestions && enhancedError.suggestions.length > 0) {
                console.error(`    Suggestions:`);
                for (const suggestion of enhancedError.suggestions) {
                  console.error(`      - ${suggestion.message}`);
                  if (suggestion.code_suggestion) {
                    console.error(`        try: ${suggestion.code_suggestion}`);
                  }
                }
              }
            } else {
              // Basic error display fallback
              console.error(`  Error: ${error.message}`);
              if ((error as any).sourceLocation) {
                console.error(`    at ${(error as any).sourceLocation}`);
              }
            }
          }
        }
      }
    } catch (error) {
      const duration = Date.now() - startTime;
      results.push({ file: inputFile, success: false, duration });
      console.log(uiError(`${inputFile} failed`));

      // Handle different error types
      if (error instanceof Error) {
        console.error(`  Error: ${error.message}`);
        if (error.stack) {
          logger.debug(`Error stack: ${error.stack}`);
        }
      } else {
        console.error(`  Error: ${String(error)}`);
      }
    }
  }

  // Summary
  const successful = results.filter(r => r.success).length;
  const failed = results.length - successful;
  const totalTime = results.reduce((sum, r) => sum + r.duration, 0);

  // Only show summary for multiple files or if verbose
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

/**
 * Compile a single file using 5IVE SDK
 */
async function compileSingleFile(
  inputFile: string,
  options: any,
  context: CommandContext,
  projectContext?: Awaited<ReturnType<typeof loadProjectConfig>>,
  sourceFiles?: string[]
): Promise<any> {
  const { logger } = context;

  // Read source code
  const sourceCode = await readFile(inputFile, 'utf8');

  // Determine output file - default to .five format
  const outputFile =
    options.output ||
    getDefaultOutputPath(
      inputFile,
      options.target || projectContext?.config.target || 'vm',
      true,
      projectContext?.config,
      projectContext?.rootDir
    );

  // Ensure output directory exists
  await mkdir(dirname(outputFile), { recursive: true });

  // Prepare SDK compilation options
  const compilationOptions: CompilationOptions = {
    optimize: parseOptimizationLevel(options.optimize),
    debug: Boolean(options.debug),
    optimizationLevel: 'production',
    includeMetrics: Boolean(options.metricsOutput),
    metricsFormat: options.metricsFormat || 'json',
    errorFormat: options.errorFormat || 'terminal',
    comprehensiveMetrics: Boolean(options.comprehensiveMetrics),
    metricsOutput: options.metricsOutput,
    flatNamespace: Boolean(options.flatNamespace)
  };

  // Compile using 5IVE SDK (includes .five format generation)
  // Compile using FiveCompilerWasm directly for rich errors
  const { FiveCompilerWasm } = await import('../wasm/compiler.js');
  const wasmCompiler = new FiveCompilerWasm(logger);
  await wasmCompiler.initialize();

  const result = await wasmCompiler.compile(sourceCode, compilationOptions);

  // Construct fiveFile object if compilation succeeded (since FiveSDK usually provides this)
  if (result.success && !result.fiveFile && result.bytecode) {
    const bytecodeBytes = result.bytecode instanceof Uint8Array
      ? result.bytecode
      : new Uint8Array(result.bytecode);
    const bytecodeBase64 = Buffer.from(bytecodeBytes).toString('base64');

    result.fiveFile = {
      bytecode: bytecodeBase64,
      abi: result.abi || { functions: {}, fields: [], version: '1.0' },
      version: '1.0',
      metadata: result.metadata || {}
    };
  }

  // Write metrics report if requested
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

  // Write output file if successful
  if (result.success) {
    if (options.debug) {
      console.log('Debug: outputFile =', outputFile);
      console.log('Debug: result.fiveFile exists =', !!result.fiveFile);
      console.log('Debug: endsWith .five =', outputFile.endsWith('.five'));
    }

    if (outputFile.endsWith('.five') && result.fiveFile) {
      // Write .five format (default)
      await writeFile(outputFile, JSON.stringify(result.fiveFile, null, 2));
    } else if (result.bytecode) {
      // Write raw bytecode (.bin format)
      await writeFile(outputFile, result.bytecode);
    }

    // Generate separate ABI if requested
    if (options.abi && result.metadata) {
      const abiFile = options.abi;
      await writeFile(abiFile, JSON.stringify(result.metadata, null, 2));
      logger.info(`ABI written to ${abiFile}`);
    }

    // Emit manifest when project config is present
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

    // Auto-generate Types
    if (result.metadata || result.abi) {
      try {
        const abi = result.metadata || result.abi;
        const generator = new TypeGenerator(abi);
        const typeDefs = generator.generate();
        // Generate .d.ts path: replace extension with .d.ts
        // If output is .five or .bin, strip and add .d.ts
        const typeFile = outputFile.replace(/(\.five|\.bin|\.fbin)$/, '') + '.d.ts';
        await writeFile(typeFile, typeDefs);
        if (context.options.verbose) logger.info(`Types generated at ${typeFile}`);
      } catch (err) {
        logger.warn(`Failed to generate types: ${err}`);
      }
    }
  }

  // Perform bytecode analysis if requested
  if (options.analyze && result.success && result.bytecode) {
    try {
      const validation = await FiveSDK.validateBytecode(result.bytecode, { debug: true });
      displayBytecodeAnalysis({ validation }, logger);
    } catch (error) {
      logger.warn(`Failed to analyze bytecode: ${error}`);
    }
  }

  // Convert result format for CLI compatibility
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

/**
 * Compile a multi-file project (entry + modules) in a single invocation.
 */
async function compileMultiProject(
  inputFiles: string[],
  options: any,
  context: CommandContext,
  projectContext: Awaited<ReturnType<typeof loadProjectConfig>> | null
): Promise<void> {
  const { logger } = context;

  if (!projectContext) {
    throw new Error('Project context is required for multi-file compilation');
  }

  const entryPoint = projectContext.config.entryPoint;
  if (!entryPoint) {
    throw new Error('multi_file_mode is enabled but entry_point is not set in five.toml');
  }

  const absoluteEntry = join(projectContext.rootDir, entryPoint);
  const allFiles = inputFiles.length > 0 ? inputFiles : [absoluteEntry];
  const sources = allFiles.some((f) => resolve(f) === resolve(absoluteEntry))
    ? allFiles
    : [absoluteEntry, ...allFiles];

  const mainSource = await readFile(absoluteEntry, 'utf8');
  const modules = await Promise.all(
    sources
      .filter((f) => resolve(f) !== resolve(absoluteEntry))
      .map(async (file) => ({
        name: basename(file, '.v'),
        source: await readFile(file, 'utf8')
      }))
  );

  const compilationOptions: CompilationOptions = {
    optimize: parseOptimizationLevel(options.optimize),
    debug: Boolean(options.debug),
    optimizationLevel: 'production',
    includeMetrics: Boolean(options.metricsOutput),
    metricsFormat: options.metricsFormat || 'json',
    errorFormat: options.errorFormat || 'terminal',
    comprehensiveMetrics: Boolean(options.comprehensiveMetrics),
    metricsOutput: options.metricsOutput,
    target: options.target || projectContext.config.target || 'vm',
    flatNamespace: Boolean(options.flatNamespace)
  };

  const outputFile =
    options.output ||
    getDefaultOutputPath(
      absoluteEntry,
      options.target || projectContext?.config.target || 'vm',
      true,
      projectContext?.config,
      projectContext?.rootDir
    );

  let result: any;
  const isTestEnv = process.env.NODE_ENV === 'test' || process.env.JEST_WORKER_ID !== undefined;

  if (context.options.verbose) {
    logger.info(`Compiling multi-file project (${sources.length} files)...`);
  }

  if (isTestEnv) {
    result = await FiveSDK.compileModules(mainSource, modules, compilationOptions);
  } else {
    // Use WASM compiler directly for multi-file compilation (like five-frontend-2)
    const { FiveCompilerWasm } = await import('../wasm/compiler.js');
    const wasmCompiler = new FiveCompilerWasm(logger);
    await wasmCompiler.initialize();

    try {
      // Call compileModules which internally uses compile_multi
      result = await wasmCompiler.compileModules(
        mainSource,
        modules,
        compilationOptions,
        isAbsolute(entryPoint) ? basename(entryPoint) : entryPoint
      );
    } catch (err: any) {
      // If multi-file fails, fall back to FiveSDK
      if (context.options.verbose) {
        logger.warn(`Multi-file compilation failed: ${err.message}, falling back to SDK...`);
      }
      result = await FiveSDK.compileModules(mainSource, modules, compilationOptions);
    }
  }

  if (context.options.verbose) {
    logger.debug(`Compilation result: success=${result.success}, has_bytecode=${!!result.bytecode}, has_abi=${!!result.abi}, errors=${result.errors ? result.errors.length : 0}`);
  }

  await mkdir(dirname(outputFile), { recursive: true });

  if (result.success) {
    // Construct fiveFile object from compilation result if needed
    if (!result.fiveFile && result.bytecode) {
      const bytecodeBytes = result.bytecode instanceof Uint8Array
        ? result.bytecode
        : new Uint8Array(result.bytecode);
      const bytecodeBase64 = Buffer.from(bytecodeBytes).toString('base64');

      result.fiveFile = {
        bytecode: bytecodeBase64,
        abi: result.abi || { functions: {}, fields: [], version: '1.0' },
        version: '1.0',
        metadata: result.metadata || {}
      };
    }

    if (outputFile.endsWith('.five') && result.fiveFile) {
      await writeFile(outputFile, JSON.stringify(result.fiveFile, null, 2));
    } else if (result.bytecode) {
      const bytecodeBytes = result.bytecode instanceof Uint8Array
        ? result.bytecode
        : new Uint8Array(result.bytecode);
      await writeFile(outputFile, Buffer.from(bytecodeBytes));
    }

    if (options.abi && result.metadata) {
      await writeFile(options.abi, JSON.stringify(result.metadata, null, 2));
      logger.info(`ABI written to ${options.abi}`);
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

    // Auto-generate Types
    if (result.metadata || result.abi) {
      try {
        const abi = result.metadata || result.abi;
        const generator = new TypeGenerator(abi);
        const typeDefs = generator.generate();
        // Generate .d.ts path: replace extension with .d.ts
        // If output is .five or .bin, strip and add .d.ts
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
        : Buffer.from(result.bytecode ?? []);
    const manifest = {
      artifact_path: outputFile,
      abi_path: options.abi,
      compiler_version: 'unknown',
      source_files: sources.map((f) => (isAbsolute(f) ? f : resolve(projectContext.rootDir, f))),
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

    console.log(uiSuccess(`build (${modules.length + 1} files, ${artifactBuffer.length} bytes)`));
  } else {
    console.log(uiError('build failed'));
    if (context.options.verbose) {
      logger.debug(`formattedErrorsTerminal prop: ${(result as any).formattedErrorsTerminal ? (result as any).formattedErrorsTerminal.substring(0, 200) : 'EMPTY/UNDEFINED'}`);
    }
    if ((result as any).formattedErrorsTerminal) {
      console.log((result as any).formattedErrorsTerminal);
    } else if (result.errors) {
      result.errors.forEach((err: any) => {
        const errorMsg = typeof err === 'string' ? err : (err.message || err.code || JSON.stringify(err) || 'Unknown error');
        console.error(`  Error: ${errorMsg}`);
      });
    }
    if (context.options.verbose && result.errors) {
      logger.debug(`Full compilation errors: ${JSON.stringify(result.errors, null, 2)}`);
    }
    throw new Error('Multi-file build failed');
  }
}

/**
 * Watch files for changes and recompile
 */
async function watchAndCompile(
  inputFiles: string[],
  options: any,
  context: CommandContext,
  projectContext?: Awaited<ReturnType<typeof loadProjectConfig>>
): Promise<void> {
  const { logger } = context;

  // Import chokidar dynamically for file watching
  const chokidar = await import('chokidar');

  logger.info('Watching for file changes...');

  // Initial compilation
  await compileFiles(inputFiles, { ...options, watch: false }, context, projectContext);

  // Watch for changes
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

  // Handle graceful shutdown
  process.on('SIGINT', () => {
    logger.info('Stopping file watcher...');
    watcher.close();
    process.exit(0);
  });
}

/**
 * Get default output path based on input file and target
 */
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

  // Use .five format by default, .bin for legacy
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

/**
 * Parse optimization level from command line option
 */
function parseOptimizationLevel(optimize: any): boolean {
  if (optimize === false || optimize === 'false' || optimize === '0') {
    return false;
  }
  return true;
}

/**
 * Display bytecode analysis using real analyzer results
 */
function displayBytecodeAnalysis(analysis: any, logger: any): void {
  console.log('\n' + section('Bytecode Analysis'));

  if (analysis.summary) {
    console.log(`  Total size: ${analysis.summary.total_size} bytes`);
    console.log(`  Instructions: ${analysis.summary.total_instructions}`);
    console.log(`  Compute units: ${analysis.summary.total_compute_units}`);
    console.log(`  Functions: ${analysis.summary.has_function_calls ? 'Yes' : 'No'}`);
    console.log(`  Jumps: ${analysis.summary.has_jumps ? 'Yes' : 'No'}`);
  }

  if (analysis.stack_analysis) {
    console.log(`  Max stack depth: ${analysis.stack_analysis.max_stack_depth}`);
    console.log(`  Stack consistency: ${analysis.stack_analysis.is_consistent ? 'Valid' : 'Invalid'}`);
  }

  if (analysis.control_flow && analysis.control_flow.basic_blocks) {
    console.log(`  Basic blocks: ${analysis.control_flow.basic_blocks.length}`);
  }

  // Show top 5 most frequent instructions
  if (analysis.instructions && analysis.instructions.length > 0) {
    console.log('\n' + section('Instruction Breakdown'));
    const instructionCounts = new Map();

    for (const inst of analysis.instructions) {
      const count = instructionCounts.get(inst.name) || 0;
      instructionCounts.set(inst.name, count + 1);
    }

    const sorted = Array.from(instructionCounts.entries())
      .sort((a, b) => b[1] - a[1])
      .slice(0, 5);

    for (const [name, count] of sorted) {
      console.log(`  ${name}: ${count} times`);
    }
  }
}

/**
 * Display opcode usage analysis
 */
function displayOpcodeAnalysis(opcodeUsage: any, opcodeAnalysis: any, logger: any): void {
  console.log('\n' + section('Opcode Usage Analysis'));

  if (opcodeUsage) {
    console.log(`  Total opcodes generated: ${opcodeUsage.total_opcodes}`);
    console.log(`  Unique opcodes used: ${opcodeUsage.unique_opcodes}`);

    if (opcodeUsage.top_opcodes && opcodeUsage.top_opcodes.length > 0) {
      console.log('\n' + section('Top Used Opcodes'));
      opcodeUsage.top_opcodes.slice(0, 5).forEach(([opcode, count]: [string, number], index: number) => {
        console.log(`    ${index + 1}. ${opcode}: ${count} times`);
      });
    }

    if (opcodeUsage.category_distribution) {
      console.log('\n' + section('Opcode Categories'));
      Object.entries(opcodeUsage.category_distribution).forEach(([category, count]) => {
        console.log(`    ${category}: ${count} opcodes`);
      });
    }
  }

  if (opcodeAnalysis && opcodeAnalysis.summary) {
    console.log('\n' + section('Comprehensive Opcode Analysis'));
    console.log(`  Available opcodes in 5IVE VM: ${opcodeAnalysis.summary.total_opcodes_available}`);
    console.log(`  Opcodes used by this script: ${opcodeAnalysis.summary.opcodes_used}`);
    console.log(`  Opcodes unused: ${opcodeAnalysis.summary.opcodes_unused}`);
    console.log(`  Usage percentage: ${opcodeAnalysis.summary.usage_percentage.toFixed(1)}%`);

    if (opcodeAnalysis.used_opcodes && opcodeAnalysis.used_opcodes.length > 0) {
      console.log('\n' + section('Used Opcodes'));
      opcodeAnalysis.used_opcodes.slice(0, 10).forEach((opcode: any) => {
        console.log(`    - ${opcode.name} (used ${opcode.usage_count} times)`);
      });

      if (opcodeAnalysis.used_opcodes.length > 10) {
        console.log(`    ... and ${opcodeAnalysis.used_opcodes.length - 10} more`);
      }
    }

    if (opcodeAnalysis.unused_opcodes && opcodeAnalysis.unused_opcodes.length > 0) {
      console.log('\n' + section('Sample Unused Opcodes'));
      opcodeAnalysis.unused_opcodes.slice(0, 8).forEach((opcode: any) => {
        console.log(`    - ${opcode.name}`);
      });

      if (opcodeAnalysis.unused_opcodes.length > 8) {
        console.log(`    ... and ${opcodeAnalysis.unused_opcodes.length - 8} more unused opcodes`);
      }
    }
  }
}
