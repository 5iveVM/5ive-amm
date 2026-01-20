"use client";

import { useIdeStore } from "@/stores/ide-store";
import Link from "next/link";
import GlassEditor from "@/components/editor/GlassEditor";
import { GlassCard } from "@/components/ui/glass-card";
import { Play, Save, Hammer, Rocket, Cpu, Wallet, Activity, Book, Loader2, Code2, Globe, Coins, Folder, Layout, Menu, X } from "lucide-react";
import { cn } from "@/lib/utils";
import { useEffect, useState, useRef } from "react";
import AppSidebar from "@/components/ide/AppSidebar";
import EditorTabs from "@/components/editor/EditorTabs";
import { FIVE_VM_PROGRAM_ID, OnChainClient } from "@/lib/onchain-client";
import { LAMPORTS_PER_SOL, Connection, PublicKey } from "@solana/web3.js";
import { ThemeToggle } from "@/components/ui/ThemeToggle";
import { WalletProvider, useConnection, useWallet } from "@solana/wallet-adapter-react";
import { useWalletModal } from "@solana/wallet-adapter-react-ui";
import { loadFiveWasm } from "@/lib/five-wasm-loader";
import { buildExecuteInstruction } from "@/lib/five-program-client";
import { NETWORKS } from "@/lib/network-config";

const MAINNET_RPC = "https://api.devnet.solana.com";
const RENT_PER_BYTE_LAM = 6960;
const ACCOUNT_OVERHEAD_BYTES = 128;
const VM_STATE_MIN_LEN = 56;
const VM_STATE_DEPLOY_FEE_OFFSET = 40;
const VM_STATE_EXECUTE_FEE_OFFSET = 44;

// Helper for VLE encoding (simplified for u32)
function encodeVLE(value: number): Uint8Array {
  const bytes: number[] = [];
  do {
    let byte = value & 0x7f;
    value >>>= 7;
    if (value !== 0) {
      byte |= 0x80;
    }
    bytes.push(byte);
  } while (value !== 0);
  return new Uint8Array(bytes);
}

/**
 * Custom Wallet Button to match the "Text Link" aesthetic of the main page.
 * Bypasses the strict styling of WalletMultiButton.
 */
function ConnectWalletButton() {
  const { setVisible } = useWalletModal();
  const { connected, publicKey, disconnect } = useWallet();

  const handleClick = () => {
    if (connected) {
      disconnect();
    } else {
      setVisible(true);
    }
  };

  const label = connected && publicKey
    ? `${publicKey.toBase58().slice(0, 4)}...${publicKey.toBase58().slice(-4)}`
    : "Wallet";

  return (
    <button
      onClick={handleClick}
      className="bg-transparent border-none shadow-none p-0 h-auto font-medium text-sm text-rose-pine-muted hover:text-rose-pine-text transition-colors relative group font-sans whitespace-nowrap"
    >
      <span className="relative">
        <span className="hidden sm:inline">{label}</span>
        <span className="sm:hidden"><Wallet size={18} /></span>
        <span className="absolute -bottom-1 left-0 w-0 h-[1px] bg-rose-pine-love transition-all group-hover:w-full hidden sm:block" />
      </span>
    </button>
  );
}

export default function IdePage() {
  const {
    code,
    bytecode,
    setCode,
    setBytecode,
    setAbi,
    logs,
    appendLog,
    setIsCompiling,
    isCompiling,
    isExecuting,
    setIsExecuting,
    updateVmState,
    selectedFunctionIndex,
    executionParams,
    executionAccounts,
    // VFS & Options
    files,
    activeFile,
    createFile,
    compilerOptions,
    projectConfig,
    // On-Chain
    isOnChainExecuting,
    setIsOnChainExecuting,
    deployments,
    currentFilename,
    // Cost
    estimatedCost,
    setEstimatedRent,
    setEstimatedDeployFee,
    setFeeConfig,
    solPrice,
    setEstimatedCost,
    setSolPrice
  } = useIdeStore();

  const { connection } = useConnection();
  const { publicKey, signTransaction, sendTransaction } = useWallet();

  const [hasUnsavedChanges, setHasUnsavedChanges] = useState(false);
  // WASM module references (useRef to avoid state proxying issues with native objects)
  const compilerRef = useRef<any>(null);
  const vmRef = useRef<any>(null);
  const paramEncoderRef = useRef<any>(null);
  const wasmModuleRef = useRef<any>(null);

  // -- New Mobile/Sidebar State --
  const [sidebarTab, setSidebarTab] = useState<'files' | 'run' | 'deploy' | 'examples'>('files');
  const [isSidebarOpen, setIsSidebarOpen] = useState(true);
  const [isSystemReady, setIsSystemReady] = useState(false);

  // Load WASM modules
  useEffect(() => {
    const loadWasm = async () => {
      try {
        appendLog('Loading Five system...', 'system');
        // Dynamic import to avoid SSR issues with WASM
        const wasm = await loadFiveWasm();
        if (typeof wasm.default === 'function') {
          try {
            await wasm.default();
          } catch (initErr) {
            console.warn("WASM init failed or already initialized:", initErr);
          }
        }
        wasmModuleRef.current = wasm;

        if (wasm.WasmFiveCompiler) {
          try {
            compilerRef.current = new wasm.WasmFiveCompiler();
            console.log("Compiler initialized");
          } catch (e) {
            console.error("Compiler init failed:", e);
          }
        } else {
          console.error("WasmFiveCompiler missing in exports");
        }

        if (wasm.ParameterEncoder) {
          paramEncoderRef.current = wasm.ParameterEncoder;
        } else {
          console.warn("ParameterEncoder missing");
        }

        appendLog('Five system ready.', 'success');
        setIsSystemReady(true);
      } catch (err) {
        console.error('Failed to load WASM:', err);
        appendLog(`Failed to load Five system: ${err}`, 'error');
        // Even on error, we might want to unblock UI or show error state
        setIsSystemReady(true);
      }
    };
    loadWasm();

    // Hydration Fix: Sync code from active file if mismatch occurs on load
    const state = useIdeStore.getState();
    if (state.activeFile && state.files[state.activeFile]) {
      const fileContent = state.files[state.activeFile];
      // If code is default "New Project" but file content is different (e.g. from persistence), sync it.
      if (state.code.includes('print("Hello 5IVE!")') && !fileContent.includes('print("Hello 5IVE!")')) {
        console.log("Hydrating code from file content...");
        setCode(fileContent);
      }
    }
  }, [appendLog, setCode]);

  // Fetch SOL Price
  useEffect(() => {
    const fetchPrice = async () => {
      try {
        const response = await fetch('https://api.coingecko.com/api/v3/simple/price?ids=solana&vs_currencies=usd');
        const data = await response.json();
        if (data.solana?.usd) {
          setSolPrice(data.solana.usd);
        }
      } catch (e) {
        console.warn("Failed to fetch SOL price, using default", e);
      }
    };
    fetchPrice();
  }, [setSolPrice]);

  // Calculate Rent automatically when bytecode changes
  useEffect(() => {
    const calculateRent = async () => {
      if (!bytecode) {
        setEstimatedCost(null);
        setEstimatedRent(null);
        setEstimatedDeployFee(null);
        setFeeConfig(null, null, null);
        return;
      }

      const space = 64 + bytecode.length; // 64 byte header + bytecode
      let rent = 0;
      let deployFeeBps = 0;

      try {
        // Try Devnet first (same rent as mainnet)
        const estimationConnection = new Connection(MAINNET_RPC);
        rent = await estimationConnection.getMinimumBalanceForRentExemption(space);
      } catch (e) {
        // Fallback to local connection
        try {
          rent = await connection.getMinimumBalanceForRentExemption(space);
        } catch (localErr) {
          // Final fallback
          rent = (ACCOUNT_OVERHEAD_BYTES + space) * RENT_PER_BYTE_LAM;
        }
      }

      try {
        const [vmStatePda] = await PublicKey.findProgramAddress(
          [new TextEncoder().encode("vm_state")],
          FIVE_VM_PROGRAM_ID
        );
        const vmInfo = await connection.getAccountInfo(vmStatePda);
        if (vmInfo?.data && vmInfo.data.length >= VM_STATE_MIN_LEN) {
          const view = new DataView(vmInfo.data.buffer, vmInfo.data.byteOffset, vmInfo.data.byteLength);
          deployFeeBps = view.getUint32(VM_STATE_DEPLOY_FEE_OFFSET, true);
          const executeFeeBps = view.getUint32(VM_STATE_EXECUTE_FEE_OFFSET, true);
          setFeeConfig(deployFeeBps, executeFeeBps, null);
        } else {
          setFeeConfig(null, null, null);
        }
      } catch (feeErr) {
        console.warn("Failed to fetch fee config, defaulting to 0 bps", feeErr);
        setFeeConfig(null, null, null);
      }

      const deployFee = Math.floor((rent * deployFeeBps) / 10000);
      const total = rent + deployFee;
      setEstimatedRent(rent);
      setEstimatedDeployFee(deployFee);
      setEstimatedCost(total);
    };
    calculateRent();
  }, [bytecode, connection, setEstimatedCost, setEstimatedDeployFee, setEstimatedRent, setFeeConfig]);


  const handleCompile = async (fileToCompile?: string) => {
    if (!compilerRef.current) return;

    // Switch to file if requested
    if (fileToCompile) {
      useIdeStore.getState().openFile(fileToCompile);
      // Small delay to allow UI to update (optional but good for visual feedback)
      await new Promise(resolve => setTimeout(resolve, 50));
    }

    setIsCompiling(true);
    appendLog('Compiling...', 'info');

    try {
      // Access fresh state directly to ensure we have the latest after generic updates
      const state = useIdeStore.getState();
      const currentCode = state.code;
      const currentActiveFile = state.activeFile;
      const currentFiles = state.files;
      const optionsConfig = state.compilerOptions;

      // 1. Create Options from Store (some fields are skipped in WASM binding, use builders where needed)
      let options = new wasmModuleRef.current.WasmCompilationOptions();

      // Direct property assignment for public fields
      options.v2_preview = optionsConfig.v2Preview;
      options.enhanced_errors = optionsConfig.enhancedErrors;
      options.include_metrics = optionsConfig.includeMetrics;
      options.enable_constraint_cache = optionsConfig.enableConstraintCache;
      options.include_debug_info = optionsConfig.includeDebugInfo;

      // Builder pattern for skipped fields (optimization_level is skipped)
      if (optionsConfig.optimizationLevel) {
        options = options.with_optimization_level(optionsConfig.optimizationLevel);
      }

      let result;

      // 2. Prepare Modules for Compilation
      let moduleFiles: [string, string][] = [];
      let entryPointFile: string | null = null;
      let mainCode = state.code;

      // PARITY: Use projectConfig if available
      if (projectConfig) {
        const configEntryPoint = projectConfig.entryPoint || projectConfig.project?.entry_point || projectConfig.build?.entry_point;

        // If modules are defined (explicit multi-file)
        if (projectConfig.modules && Object.keys(projectConfig.modules).length > 0) {
          const allModuleFiles = new Set<string>();

          // Collect all files from all modules
          Object.values(projectConfig.modules).forEach((modFiles: any) => {
            if (Array.isArray(modFiles)) {
              modFiles.forEach(f => allModuleFiles.add(f));
            }
          });

          if (configEntryPoint) {
            entryPointFile = configEntryPoint;
            allModuleFiles.add(configEntryPoint);
          } else {
            // Fallback to active file if part of modules, otherwise error?
            // For IDE friendliness, we default to active file.
            entryPointFile = state.activeFile;
          }

          if (entryPointFile) {
            // Filter files in VFS that match the config
            moduleFiles = Object.entries(currentFiles).filter(([name]) =>
              allModuleFiles.has(name) && name !== entryPointFile
            );
          }
        }
        // If no modules but entry point is set (auto discovery or single file)
        else if (configEntryPoint) {
          entryPointFile = configEntryPoint;

          // Auto-discovery logic (grab all .v files)
          const sourceFiles = Object.entries(currentFiles).filter(([name]) =>
            name !== 'five.toml' && name.endsWith('.v') && name !== entryPointFile
          );
          moduleFiles = sourceFiles;
        }
      }

      // Fallback to legacy loose mode if no config or no entry point resolved
      if (!entryPointFile) {
        entryPointFile = state.activeFile;

        if (entryPointFile) {
          moduleFiles = Object.entries(currentFiles).filter(([name]) =>
            name !== 'five.toml' && name !== 'Five.toml' && name.endsWith('.v') && name !== entryPointFile
          );
        }
      }

      // Determine content for entry point
      // If we are editing the entry point, use the live code from the editor (state.code)
      // Otherwise load from VFS
      if (entryPointFile) {
        if (entryPointFile === state.activeFile) {
          mainCode = state.code;
        } else if (currentFiles[entryPointFile]) {
          mainCode = currentFiles[entryPointFile];
        } else {
          // file missing?
          appendLog(`Entry point ${entryPointFile} not found`, 'error');
          setIsCompiling(false);
          return;
        }
      } else {
        // Should not happen if activeFile is set
        appendLog(`No active file or entry point to compile`, 'error');
        setIsCompiling(false);
        return;
      }

      const totalSourceFiles = moduleFiles.length + 1;

      if (totalSourceFiles > 1) {
        const fileNames = moduleFiles.map(([name]) => name).join(', ');
        appendLog(`Compiling multi-file project (${totalSourceFiles} files)...`, 'system');
        const modules = moduleFiles.map(([name, content]) => ({ name, source: content }));
        // Use compile_multi
        result = compilerRef.current.compile_multi(mainCode, modules, options);
      } else {
        // Single file mode
        result = compilerRef.current.compile(mainCode, options);
      }

      // Properties are getters in wasm-bindgen classes
      const success = result.success;

      if (success) {
        // Try getter first, then method
        const bytes = result.bytecode || (typeof result.get_bytecode === 'function' ? result.get_bytecode() : null);

        if (bytes) {
          setBytecode(bytes);
          appendLog(`Compilation successful! (${bytes.length} bytes)`, 'success');

          // Switch to Run Tab automatically
          setSidebarTab('run');

          // --- Artifact Generation ---
          try {
            const hex = Array.from(bytes).map((b: any) => b.toString(16).padStart(2, '0')).join('');
            const baseName = state.currentFilename ? state.currentFilename.replace(/\.five$/, '').replace(/\.v$/, '') : 'program';
            createFile(`target/deploy/${baseName}.bin.hex`, hex, false);
            // Initialize VM with new bytecode
            const newVm = new wasmModuleRef.current.FiveVMWasm(bytes);
            vmRef.current = newVm;
          } catch (artifactErr) {
            console.warn("Failed to write bytecode artifact:", artifactErr);
          }
        } else {
          appendLog('Compilation success reported but bytecode missing', 'error');
        }


        // Extract ABI if available
        try {
          const abiJson = result.abi || (typeof result.get_abi === 'function' ? result.get_abi() : null);
          // Handle both string and object ABI formats (WASM may return either)
          let abi = abiJson ? (typeof abiJson === 'string' ? JSON.parse(abiJson) : abiJson) : null;

          // Helper to extract names from source using regex
          const extractNamesFromSource = (source: string) => {
            const names: string[] = [];
            // Matches 'pub name(' pattern
            const regex = /\bpub\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\(/g;
            let match;
            while ((match = regex.exec(source)) !== null) {
              names.push(match[1]);
            }
            return names;
          };

          const sourceNames = extractNamesFromSource(state.code);

          // Fallback/Augment: Extract metadata from bytecode directly
          try {
            const metadata = compilerRef.current.extractFunctionMetadata(bytes);

            if (metadata && metadata.length > 0) {
              if (!abi) { abi = { functions: [], types: [] }; }

              metadata.forEach((meta: any) => {
                const existingIndex = abi.functions.findIndex((f: any) => f.index === meta.address);
                if (existingIndex === -1) {
                  abi.functions.push({
                    name: meta.name,
                    index: meta.address,
                    parameters: Array(meta.param_count).fill({ name: "arg", type: "Value" })
                  });
                } else {
                  if (!abi.functions[existingIndex].name || abi.functions[existingIndex].name.toLowerCase().startsWith("ref")) {
                    abi.functions[existingIndex].name = meta.name;
                  }
                }
              });
              appendLog(`Extracted metadata for ${metadata.length} functions`, 'info');
            }
          } catch (metaErr) {
            console.warn("Metadata extraction failed, falling back to source parsing:", metaErr);
            if (sourceNames.length > 0) {
              if (!abi) {
                abi = { functions: [], types: [] };
              } else if (abi.functions && !Array.isArray(abi.functions)) {
                abi.functions = Object.values(abi.functions);
              } else if (!abi.functions) {
                abi.functions = [];
              }

              sourceNames.forEach((name, idx) => {
                const existingIndex = abi.functions.findIndex((f: any) => f.index === idx);
                if (existingIndex === -1) {
                  abi.functions.push({ name: name, index: idx, parameters: [] });
                } else {
                  if (!abi.functions[existingIndex].name || abi.functions[existingIndex].name.toLowerCase().startsWith("ref")) {
                    abi.functions[existingIndex].name = name;
                  }
                }
              });
              appendLog(`Recovered ${sourceNames.length} function names from source`, 'warning');
            }
          }

          if (!abi && sourceNames.length > 0) {
            abi = { functions: [], types: [] };
            sourceNames.forEach((name, idx) => {
              abi.functions.push({ name: name, index: idx, parameters: [] });
            });
            appendLog(`Built ABI from source code (${sourceNames.length} functions)`, 'warning');
          }

          if (abi) {
            setAbi(abi);
            appendLog('ABI loaded successfully', 'info');
            try {
              const baseName = state.currentFilename ? state.currentFilename.replace(/\.five$/, '').replace(/\.v$/, '') : 'program';
              createFile(`target/deploy/${baseName}.abi.json`, JSON.stringify(abi, null, 2), false);
            } catch (abiArtifactErr) {
              console.warn("Failed to write ABI artifact:", abiArtifactErr);
            }
          }
        } catch (err: any) {
          console.error("Compilation failed:", err);
          setIsCompiling(false);

          let errorMsg = "Internal compilation error";
          if (err.message) {
            errorMsg = err.message;
          } else if (typeof err === "string") {
            errorMsg = err;
          } else if (typeof err === "object") {
            try {
              errorMsg = JSON.stringify(err);
            } catch {
              errorMsg = String(err);
            }
          }

          appendLog(`Compilation error: ${errorMsg}`, 'error');
        }

      } else {
        // Handle errors
        let errorMsg = "Unknown compilation error";
        if (result.errors && result.errors.length > 0) {
          errorMsg = Array.from(result.errors).join('\n');
        } else if (result.compiler_errors && result.compiler_errors.length > 0) {
          errorMsg = Array.from(result.compiler_errors).map((e: any) => {
            const msg = e.message || String(e);
            const loc = e.location ? ` at line ${e.location.line}` : '';
            return `${msg}${loc}`;
          }).join('\n');
        } else if (result.error_message) {
          errorMsg = result.error_message;
        } else {
          // Fallback for generic result failure
          try {
            const json = JSON.stringify(result);
            // If it's just successful: false with no error info, say that
            errorMsg = json === "{}" ? "Unknown compilation failure" : json;
          } catch {
            errorMsg = String(result);
          }
        }

        appendLog(`Compilation failed:\n${errorMsg}`, 'error');
      }
    } catch (err) {
      appendLog(`Compiler error: ${err}`, 'error');
    } finally {
      setIsCompiling(false);
    }
  };

  const handleRun = async () => {
    if (!bytecode) {
      appendLog("No bytecode to execute. Compile first.", "error");
      return;
    }

    setIsExecuting(true);

    try {
      // 1. Encode parameters using WASM helper
      let encodedParams = new Uint8Array([]);

      if (paramEncoderRef.current) {
        try {
          encodedParams = paramEncoderRef.current.encode_execute_vle(selectedFunctionIndex, executionParams);
        } catch (e) {
          appendLog(`Parameter encoding failed: ${e}`, 'error');
          throw e; // Rethrow to stop execution
        }
      }

      // Branch: On-Chain vs Local
      if (isOnChainExecuting) {
        if (!publicKey || !signTransaction) {
          appendLog("Wallet not connected. Connect wallet for on-chain execution.", "error");
          setIsExecuting(false);
          return;
        }

        const deployment = deployments[currentFilename || ""];
        if (!deployment) {
          appendLog(`No deployment found for ${currentFilename || "current file"}. Deploy first.`, "error");
          setIsExecuting(false);
          return;
        }

        appendLog(`Executing on-chain... Function #${selectedFunctionIndex}`, "info");

        try {
          // Get the ABI and selected function
          const abi = useIdeStore.getState().abi;
          const selectedNetwork = useIdeStore.getState().selectedNetwork || 'localnet';
          const networkConfig = NETWORKS[selectedNetwork];

          if (abi && abi.functions) {
            // Use FiveProgram for execution (preferred path)
            const functionList = Array.isArray(abi.functions) ? abi.functions : Object.values(abi.functions);
            const selectedFunc = functionList.find((f: any) => f.index === selectedFunctionIndex);

            if (!selectedFunc) {
              appendLog(`Function with index ${selectedFunctionIndex} not found in ABI`, "error");
              setIsExecuting(false);
              return;
            }

            appendLog(`Using FiveProgram to execute '${selectedFunc.name}'...`, "info");

            // Build accounts map from executionAccounts array
            const accountsMap: Record<string, string> = {};
            const accountParams = (selectedFunc.parameters || []).filter((p: any) => p.is_account);
            accountParams.forEach((param: any, idx: number) => {
              if (executionAccounts[idx]) {
                accountsMap[param.name] = executionAccounts[idx];
              }
            });

            // Build args map from executionParams array
            const argsMap: Record<string, any> = {};
            const dataParams = (selectedFunc.parameters || []).filter((p: any) => !p.is_account);
            dataParams.forEach((param: any, idx: number) => {
              if (executionParams[idx] !== undefined) {
                argsMap[param.name] = executionParams[idx];
              }
            });

            // Build instruction using FiveProgram
            const { instruction } = await buildExecuteInstruction({
              network: selectedNetwork,
              scriptAccount: deployment.scriptAccount,
              abi: abi,
              functionName: selectedFunc.name,
              accounts: accountsMap,
              args: argsMap,
              debug: true
            });

            // Create and send transaction
            const { Transaction } = await import('@solana/web3.js');
            const deployConnection = new Connection(networkConfig.rpcUrl, 'confirmed');

            const transaction = new Transaction();
            transaction.add(instruction);
            transaction.feePayer = publicKey;

            const latestBlockhash = await deployConnection.getLatestBlockhash();
            transaction.recentBlockhash = latestBlockhash.blockhash;

            let signature: string;
            if (signTransaction) {
              try {
                const signedTx = await signTransaction(transaction);
                signature = await deployConnection.sendRawTransaction(signedTx.serialize(), {
                  skipPreflight: true
                });
              } catch (err: any) {
                if (err.toString().includes("signTransaction is not a function") && sendTransaction) {
                  console.warn("signTransaction failed, falling back to sendTransaction");
                  signature = await sendTransaction(transaction, deployConnection);
                } else {
                  throw err;
                }
              }
            } else if (sendTransaction) {
              signature = await sendTransaction(transaction, deployConnection);
            } else {
              throw new Error("Wallet does not support signing or sending transactions");
            }

            await deployConnection.confirmTransaction(signature, 'confirmed');

            appendLog(`On-Chain Execution Successful!`, "success");
            appendLog(`Tx: ${signature}`, "success");

          } else {
            // Fallback to OnChainClient if no ABI (legacy path)
            appendLog(`No ABI available, using legacy execution...`, "warning");

            const client = new OnChainClient(connection, {
              publicKey,
              signTransaction,
              sendTransaction
            });

            const result = await client.execute(
              deployment.scriptAccount,
              selectedFunctionIndex,
              encodedParams
            );

            if (result.success) {
              appendLog(`On-Chain Execution Successful!`, "success");
              appendLog(`Tx: ${result.transactionId}`, "success");
            } else {
              appendLog(`On-Chain Execution Failed: ${result.error}`, "error");
              if (result.logs && result.logs.length > 0) {
                result.logs.forEach(l => appendLog(`${l}`, "info"));
              }
            }
          }

        } catch (e) {
          appendLog(`On-chain execution error: ${e}`, "error");
        }

      } else {
        // Local VM Simulation
        if (!vmRef.current && wasmModuleRef.current) {
          try {
            vmRef.current = new wasmModuleRef.current.FiveVMWasm(bytecode);
          } catch (e) {
            console.error("Failed to init VM:", e);
          }
        }

        // If still no VM, try to recreate it
        if (!vmRef.current && wasmModuleRef.current) {
          vmRef.current = new wasmModuleRef.current.FiveVMWasm(bytecode);
        }

        if (!vmRef.current) {
          appendLog("VM not initialized", "error");
          return;
        }

        appendLog(`Executing locally... Function #${selectedFunctionIndex}`, 'info');

        // 2. Construct Execution Payload: [Discriminator(9)] + [VLE(Index)] + [EncodedParams]
        const discriminator = new Uint8Array([9]); // Execute discriminator
        const indexBytes = encodeVLE(selectedFunctionIndex);

        const payload = new Uint8Array(discriminator.length + indexBytes.length + encodedParams.length);
        payload.set(discriminator, 0);
        payload.set(indexBytes, discriminator.length);
        payload.set(encodedParams, discriminator.length + indexBytes.length);

        // Artificial Delay for UX (so user sees the spinner)
        await new Promise(resolve => setTimeout(resolve, 500));

        // 3. Execute
        const result = await vmRef.current.execute_partial(payload, []);

        // Update State
        let stack: string[] = [];
        let ip = 0;
        let cu = 0;
        let error = null;
        let memory: Uint8Array | null = null;

        try {
          stack = result.final_stack || [];
          ip = result.instruction_pointer || 0;
          cu = result.compute_units_used || 0;
          memory = result.final_memory || null;
          error = result.error_message;
        } catch (e) {
          console.warn("Error accessing result fields", e);
        }

        console.log("VM Result:", result); // Debugging

        updateVmState({
          stack: stack,
          instructionPointer: ip,
          computeUnits: cu,
          memory: memory,
        });

        if (error) {
          appendLog(`Runtime Error: ${error}`, 'error');
        } else {
          appendLog(`Execution successful! Used ${cu} CU.`, 'success');
        }
      }

    } catch (err) {
      appendLog(`Execution error: ${err}`, 'error');
    } finally {
      setIsExecuting(false);
    }
  };

  // Hydration fix
  const [mounted, setMounted] = useState(false);
  useEffect(() => {
    setMounted(true);
  }, []);

  if (!mounted) {
    return <main className="h-screen overflow-hidden bg-rose-pine-base flex items-center justify-center"><Loader2 className="animate-spin text-rose-pine-iris" /></main>;
  }

  return (
    <div className="flex flex-col min-h-[100dvh] supports-[height:100dvh]:h-[100dvh] bg-rose-pine-base text-rose-pine-text font-sans selection:bg-rose-pine-iris/20 overflow-hidden pt-16 md:pt-24">
      {/* Header */}
      <header className="fixed top-2 left-0 right-0 z-50 flex items-center justify-center px-2 md:px-4 pointer-events-none">
        <div className="pointer-events-auto flex items-center justify-between px-4 py-2 rounded-full border border-rose-pine-hl-low/20 bg-rose-pine-surface/80 backdrop-blur-2xl shadow-[0_8px_32px_rgba(0,0,0,0.12)] w-full max-w-7xl transition-all duration-500 hover:shadow-[0_8px_40px_rgba(0,0,0,0.2)] hover:border-rose-pine-hl-med/30 shrink-0 h-14 md:h-auto">
          <div className="flex items-center gap-2 sm:gap-4 flex-1 md:flex-none">
            <div className="flex items-center gap-2 sm:gap-3">
              <button
                className="md:hidden p-1.5 hover:bg-white/5 rounded-lg text-rose-pine-subtle"
                onClick={() => setIsSidebarOpen(!isSidebarOpen)}
              >
                {isSidebarOpen ? <X size={18} /> : <Menu size={18} />}
              </button>
              <Link href="/" className="font-black text-lg sm:text-xl tracking-tighter bg-gradient-to-b from-white via-[#c4a7e7] to-[#eb6f92] bg-clip-text text-transparent hover:opacity-80 transition-opacity">
                5IVE
              </Link>
              <span className="hidden sm:inline-block px-2 py-0.5 rounded-full bg-rose-pine-surface border border-rose-pine-hl-low text-[10px] font-bold uppercase tracking-wider text-rose-pine-subtle">
                IDE
              </span>
            </div>

            <div className="hidden md:block h-4 w-px bg-white/10 mx-1 sm:mx-2" />

            {/* Desktop Quick Nav */}
            <div className="hidden md:flex items-center gap-2">
              <Link
                href="/docs"
                className="p-1.5 hover:bg-white/5 rounded-lg text-rose-pine-subtle hover:text-rose-pine-text transition-colors"
                title="Documentation"
              >
                <Book size={16} />
              </Link>
              <button className="p-1.5 hover:bg-white/5 rounded-lg text-rose-pine-subtle hover:text-rose-pine-text transition-colors" title="Save">
                <Save size={16} />
              </button>
            </div>
          </div>

          {/* Navigation - Top Bar (Replaces Sidebar Icons on Tablet/Desktop) */}
          <div className="hidden md:flex items-center gap-1 bg-rose-pine-surface/50 p-1 rounded-xl border border-rose-pine-hl-low/20 mx-2">
            <button
              onClick={() => {
                if (sidebarTab === 'files' && isSidebarOpen) setIsSidebarOpen(false);
                else { setSidebarTab('files'); setIsSidebarOpen(true); }
              }}
              className={cn(
                "flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-medium transition-all duration-200",
                sidebarTab === 'files' && isSidebarOpen
                  ? "bg-rose-pine-iris text-white shadow-sm"
                  : "text-rose-pine-subtle hover:text-rose-pine-text hover:bg-white/5"
              )}
            >
              <Folder size={14} />
              <span className="hidden lg:inline">Files</span>
            </button>

            <button
              onClick={() => {
                if (sidebarTab === 'run' && isSidebarOpen) setIsSidebarOpen(false);
                else { setSidebarTab('run'); setIsSidebarOpen(true); }
              }}
              className={cn(
                "flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-medium transition-all duration-200",
                sidebarTab === 'run' && isSidebarOpen
                  ? "bg-rose-pine-iris text-white shadow-sm"
                  : "text-rose-pine-subtle hover:text-rose-pine-text hover:bg-white/5"
              )}
            >
              <Play size={14} />
              <span className="hidden lg:inline">Run</span>
            </button>

            <button
              onClick={() => {
                if (sidebarTab === 'deploy' && isSidebarOpen) setIsSidebarOpen(false);
                else { setSidebarTab('deploy'); setIsSidebarOpen(true); }
              }}
              className={cn(
                "flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-medium transition-all duration-200",
                sidebarTab === 'deploy' && isSidebarOpen
                  ? "bg-rose-pine-iris text-white shadow-sm"
                  : "text-rose-pine-subtle hover:text-rose-pine-text hover:bg-white/5"
              )}
            >
              <Rocket size={14} />
              <span className="hidden lg:inline">Deploy</span>
            </button>

            <button
              onClick={() => {
                if (sidebarTab === 'examples' && isSidebarOpen) setIsSidebarOpen(false);
                else { setSidebarTab('examples'); setIsSidebarOpen(true); }
              }}
              className={cn(
                "flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-medium transition-all duration-200",
                sidebarTab === 'examples' && isSidebarOpen
                  ? "bg-rose-pine-iris text-white shadow-sm"
                  : "text-rose-pine-subtle hover:text-rose-pine-text hover:bg-white/5"
              )}
            >
              <Book size={14} />
              <span className="hidden lg:inline">Examples</span>
            </button>
          </div>

          {/* Center Tabs - Desktop Only */}
          <div className="flex items-center justify-end gap-2 sm:gap-3 flex-1 md:flex-none">
            {/* Mobile Compile Button (Icon Only) */}
            <button
              onClick={() => handleCompile()}
              disabled={isCompiling}
              className={`
                       flex items-center justify-center p-2 rounded-lg text-xs font-medium border transition-all duration-300 md:hidden
                       ${isCompiling
                  ? 'bg-rose-pine-surface/50 border-rose-pine-hl-low text-rose-pine-subtle cursor-wait'
                  : 'bg-rose-pine-overlay/50 border-rose-pine-hl-high hover:border-rose-pine-iris/50 hover:bg-rose-pine-overlay text-rose-pine-text'
                }
                   `}
            >
              {isCompiling ? <Loader2 size={16} className="animate-spin" /> : <Hammer size={16} />}
            </button>

            {/* Desktop Compile Button */}
            <button
              onClick={() => handleCompile()}
              disabled={isCompiling}
              className={`
                       hidden md:flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-medium border transition-all duration-300
                       ${isCompiling
                  ? 'bg-rose-pine-surface/50 border-rose-pine-hl-low text-rose-pine-subtle cursor-wait'
                  : 'bg-rose-pine-overlay/50 border-rose-pine-hl-high hover:border-rose-pine-iris/50 hover:bg-rose-pine-overlay text-rose-pine-text'
                }
                   `}
            >
              {isCompiling ? <Loader2 size={14} className="animate-spin" /> : <Hammer size={14} />}
              <span className="hidden lg:inline">Compile</span>
            </button>


            {/* Theme Toggle */}
            <div className="hidden sm:block">
              <ThemeToggle />
            </div>

            <ConnectWalletButton />
          </div>
        </div>
      </header>

      {/* Main Content Area */}
      <main className="flex-1 relative overflow-hidden flex flex-col">

        {/* Floating Sidebar */}
        <AppSidebar
          isOpen={isSidebarOpen}
          onToggle={() => setIsSidebarOpen(!isSidebarOpen)}
          onCompile={handleCompile}
          activeTab={sidebarTab}
          onTabChange={setSidebarTab}
          onRun={handleRun}
          isExecuting={isExecuting}
          isOnChain={isOnChainExecuting}
          onToggleMode={setIsOnChainExecuting}
          estimatedCost={estimatedCost}
          solPrice={solPrice}
        />

        {/* Backdrop for Mobile Sidebar */}
        {isSidebarOpen && (
          <div
            className="fixed inset-0 bg-black/50 z-30 md:hidden backdrop-blur-sm"
            onClick={() => setIsSidebarOpen(false)}
          />
        )}

        {/* Editor Area (Full Background) */}
        <div className={cn(
          "absolute inset-0 transition-all duration-300 pointer-events-none md:pointer-events-auto",
          isSidebarOpen ? "md:pl-96" : "md:pl-16" // Only push padding on Medium+ screens
        )}>
          <div className="flex flex-col h-full px-0 pb-0 sm:px-4 sm:pb-4 relative pointer-events-auto">
            {/* Loading Overlay */}
            {!isSystemReady && (
              <div className="absolute inset-0 z-50 flex items-center justify-center bg-rose-pine-base/80 backdrop-blur-sm rounded-2xl animate-in fade-in duration-500">
                <div className="flex flex-col items-center gap-4">
                  <div className="relative">
                    <div className="absolute inset-0 bg-rose-pine-iris/20 blur-xl rounded-full animate-pulse" />
                    <Loader2 size={48} className="text-rose-pine-iris animate-spin relative z-10" />
                  </div>
                  <p className="text-rose-pine-muted font-mono text-sm tracking-widest uppercase animate-pulse">Initializing System...</p>
                </div>
              </div>
            )}

            {/* Floating Tabs Area */}
            <div className={`flex-shrink-0 mb-0 sm:mb-4 sm:pl-0 transition-opacity duration-500 ${!isSystemReady ? 'opacity-0' : 'opacity-100'}`}>
              <EditorTabs />
            </div>

            {/* Editor Surface - Made fully rounded and floating */}
            <div className="flex-1 overflow-hidden glass-panel border-white/5 border shadow-2xl relative md:rounded-2xl rounded-none border-x-0 border-b-0">
              <GlassEditor />
            </div>
          </div>
        </div>

      </main>
    </div>
  );
}
