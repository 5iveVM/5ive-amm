"use client";

import { useState, useEffect } from "react";
import { useWallet, useConnection } from "@solana/wallet-adapter-react";
import { Connection } from "@solana/web3.js";
import { useIdeStore } from "@/stores/ide-store";
import { OnChainClient } from "@/lib/onchain-client";
import { NETWORKS, type NetworkType, getExplorerUrl } from "@/lib/network-config";
import { GlassCard, GlassHeader } from "@/components/ui/glass-card";
import { Rocket, Loader2, CheckCircle, XCircle, AlertCircle, Copy, Globe, Coins, ChevronDown } from "lucide-react";
import { LAMPORTS_PER_SOL } from "@solana/web3.js";

export default function DeployManager() {
    const { connection } = useConnection();
    const { publicKey, signTransaction, sendTransaction } = useWallet();
    const {
        bytecode,
        abi,
        appendLog,
        addDeployment,
        currentFilename,
        code,
        rpcEndpoint,
        setRpcEndpoint,
        estimatedCost,
        estimatedRent,
        estimatedDeployFee,
        deployFeeLamports,
        solPrice,
        selectedNetwork,
        setSelectedNetwork
    } = useIdeStore();

    const [isDeploying, setIsDeploying] = useState(false);
    const [deploymentResult, setDeploymentResult] = useState<{ signature: string; scriptAccount: string } | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [isEditingRpc, setIsEditingRpc] = useState(false);

    const handleDeploy = async () => {
        if (!bytecode) return;

        if (!publicKey) {
            setError("Please connect your wallet to deploy.");
            return;
        }

        setIsDeploying(true);
        setError(null);
        setDeploymentResult(null);
        const networkConfig = NETWORKS[selectedNetwork];
        appendLog(`Preparing to deploy to ${networkConfig.name}...`, "info");

        try {
            // Create connection for the selected network
            const deployConnection = new Connection(networkConfig.rpcUrl, 'confirmed');

            // Check balance first
            const balance = await deployConnection.getBalance(publicKey);
            if (balance < 0.05 * LAMPORTS_PER_SOL) {
                throw new Error("Insufficient funds. You need at least 0.05 SOL to deploy.");
            }

            // Initialize OnChainClient with network-specific programId and both signing/sending methods
            const client = new OnChainClient(deployConnection, {
                publicKey,
                signTransaction,
                sendTransaction
            }, networkConfig.programId);

            // Execute deployment
            const result = await client.deploy(bytecode);

            if (!result.success || !result.scriptAccount || !result.transactionId) {
                throw new Error(result.error || "Deployment failed with unknown error");
            }

            // Success!
            setDeploymentResult({
                signature: result.transactionId,
                scriptAccount: result.scriptAccount
            });

            appendLog(`Deployment successful! Script Account: ${result.scriptAccount}`, "success");

            // Save deployment record
            addDeployment(currentFilename || "untitled.five", {
                scriptAccount: result.scriptAccount,
                programId: result.scriptAccount,
                deployedAt: Date.now(),
                transactionId: result.transactionId
            });

        } catch (err: any) {
            console.error("DeployManager Error:", err);
            let errorMessage = "Deployment error";

            if (err instanceof Error) {
                errorMessage = err.message;
            } else if (typeof err === "string") {
                errorMessage = err;
            } else {
                try {
                    const json = JSON.stringify(err);
                    errorMessage = json === "{}" ? String(err) : json;
                } catch {
                    errorMessage = String(err);
                }
            }

            setError(errorMessage);
            appendLog(`Deployment failed: ${errorMessage}`, "error");
        } finally {
            setIsDeploying(false);
        }
    };

    if (!bytecode) {
        return (
            <GlassCard className="h-full flex flex-col items-center justify-center p-6 border-rose-pine-hl-low/30 relative overflow-hidden group">
                {/* Ambient Background */}
                <div className="absolute inset-0 bg-gradient-to-br from-rose-pine-love/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-700" />

                <div className="relative z-10 flex flex-col items-center text-center gap-3">
                    <div className="w-12 h-12 rounded-2xl bg-white/5 flex items-center justify-center border border-white/10 shadow-inner backdrop-blur-sm">
                        <Rocket className="text-rose-pine-subtle" size={24} />
                    </div>
                    <div>
                        <h3 className="text-rose-pine-text font-medium mb-1">Ready to Deploy?</h3>
                        <p className="text-sm text-rose-pine-subtle max-w-[200px]">
                            Compile your 5IVE DSL code to generate bytecode for deployment.
                        </p>
                    </div>
                </div>
            </GlassCard>
        );
    }

    return (
        <GlassCard className="flex flex-col h-full border-rose-pine-hl-low/50 overflow-hidden">
            <GlassHeader title="Deployment Manager" className="bg-rose-pine-base/40 backdrop-blur-xl border-b border-white/5" />

            <div className="p-5 space-y-6 overflow-y-auto custom-scrollbar flex-1">
                {/* Deployment Controls */}
                <div className="space-y-4">
                    {/* Network Selector */}
                    <div className="flex flex-col gap-2">
                        <label className="text-[10px] uppercase tracking-wider text-rose-pine-muted font-bold flex items-center gap-1.5 opacity-80">
                            <Globe size={12} className="text-rose-pine-iris" />
                            Target Network
                        </label>

                        <div className="relative">
                            <select
                                value={selectedNetwork}
                                onChange={(e) => setSelectedNetwork(e.target.value as NetworkType)}
                                className="w-full appearance-none bg-rose-pine-surface/50 border border-white/10 rounded-lg px-3 py-2.5 text-xs text-rose-pine-text focus:outline-none focus:border-rose-pine-iris/50 focus:ring-1 focus:ring-rose-pine-iris/20 transition-all cursor-pointer pr-8"
                            >
                                <option value="localnet">🟢 Localnet (127.0.0.1:8899)</option>
                                <option value="devnet">🟡 Devnet (api.devnet.solana.com)</option>
                            </select>
                            <ChevronDown size={14} className="absolute right-3 top-1/2 -translate-y-1/2 text-rose-pine-subtle pointer-events-none" />
                        </div>

                        <div className="flex items-center gap-2 text-[10px] text-rose-pine-muted">
                            <div className={`w-1.5 h-1.5 rounded-full ${selectedNetwork === 'localnet' ? 'bg-emerald-400' : 'bg-rose-pine-gold'} animate-pulse`} />
                            <span className="font-mono">{NETWORKS[selectedNetwork].rpcUrl}</span>
                        </div>
                    </div>

                    {/* Cost Estimation */}
                    <div className="bg-rose-pine-overlay/30 rounded-lg p-3 border border-white/5 flex flex-col gap-3">
                        <div className="flex items-center justify-between">
                            <div className="flex items-center gap-2">
                                <div className="p-1.5 rounded-md bg-rose-pine-gold/10 text-rose-pine-gold">
                                    <Coins size={14} />
                                </div>
                                <span className="text-[10px] text-rose-pine-muted uppercase tracking-wider font-medium">Est. Cost (Rent + Deploy Fee)</span>
                            </div>
                            <span className="text-xs text-rose-pine-text family-mono font-medium">
                                {estimatedCost ? `${(estimatedCost / LAMPORTS_PER_SOL).toFixed(5)} SOL` : 'Calculating...'}
                            </span>
                        </div>

                        {estimatedRent !== null && estimatedDeployFee !== null && (
                            <div className="flex items-center justify-between border-t border-white/5 pt-2 text-[10px] text-rose-pine-muted font-mono">
                                <span>
                                    rent {(estimatedRent / LAMPORTS_PER_SOL).toFixed(5)} ◎ + fee {(estimatedDeployFee / LAMPORTS_PER_SOL).toFixed(5)} ◎
                                    {deployFeeLamports !== null ? ` (${deployFeeLamports.toLocaleString()} lamports)` : ""}
                                </span>
                            </div>
                        )}

                        <div className="flex items-center justify-between border-t border-white/5 pt-2">
                            <span className="text-[10px] text-rose-pine-muted uppercase tracking-wider font-medium">USD Value</span>
                            <div className="text-xs text-rose-pine-subtle family-mono">
                                {estimatedCost !== null
                                    ? `~$${((estimatedCost / LAMPORTS_PER_SOL) * solPrice).toFixed(2)}`
                                    : '...'}
                            </div>
                        </div>
                    </div>

                    <button
                        onClick={handleDeploy}
                        disabled={isDeploying || estimatedCost === null}
                        className={`
                            relative w-full py-3 px-4 rounded-xl flex items-center justify-center gap-2 font-medium transition-all duration-300
                            ${isDeploying || !publicKey
                                ? "bg-rose-pine-surface/50 text-rose-pine-subtle cursor-not-allowed border border-white/5"
                                : "bg-gradient-to-r from-rose-pine-love to-rose-pine-iris text-white shadow-lg shadow-rose-pine-love/25 hover:shadow-rose-pine-love/40 hover:scale-[1.02] border border-white/10"
                            }
                        `}
                    >
                        {isDeploying ? (
                            <>
                                <Loader2 size={18} className="animate-spin" />
                                <span>Deploying Program...</span>
                            </>
                        ) : !publicKey ? (
                            <>
                                <Globe size={18} />
                                <span>Connect Wallet to Deploy</span>
                            </>
                        ) : (
                            <>
                                <Rocket size={18} />
                                <span>Deploy to Localnet</span>
                            </>
                        )}
                    </button>
                </div>

                {/* Feedback Section */}
                <div className="space-y-3">
                    {deploymentResult && (
                        <div className="p-4 rounded-xl bg-emerald-500/10 border border-emerald-500/20 backdrop-blur-sm animate-in fade-in slide-in-from-bottom-2 duration-500">
                            <div className="flex items-start gap-3">
                                <div className="p-1 rounded-full bg-emerald-500/20 text-emerald-400 mt-0.5">
                                    <CheckCircle size={16} />
                                </div>
                                <div className="flex-1 space-y-2">
                                    <div>
                                        <h4 className="text-sm font-medium text-emerald-100">Deployment Successful!</h4>
                                        <p className="text-xs text-emerald-200/70">
                                            Your program is live on Localnet.
                                        </p>
                                    </div>

                                    <div className="bg-black/20 rounded p-2 text-xs font-mono text-emerald-300 break-all select-all flex flex-col gap-1">
                                        <span className="text-[10px] text-emerald-500/70 uppercase">Script Account</span>
                                        {deploymentResult.scriptAccount}
                                    </div>

                                    <div className="flex items-center gap-2 pt-1">
                                        <span className="text-[10px] text-emerald-500/50 uppercase">Tx Signature</span>
                                        <span className="text-[10px] font-mono text-emerald-500/70 truncate max-w-[150px]">{deploymentResult.signature}</span>
                                    </div>
                                </div>
                            </div>
                        </div>
                    )}

                    {error && (
                        <div className="p-4 rounded-xl bg-rose-500/10 border border-rose-500/20 backdrop-blur-sm animate-in fade-in slide-in-from-bottom-2">
                            <div className="flex items-start gap-3">
                                <div className="p-1 rounded-full bg-rose-500/20 text-rose-400 mt-0.5">
                                    <XCircle size={16} />
                                </div>
                                <div className="flex-1">
                                    <h4 className="text-sm font-medium text-rose-100 mb-1">Deployment Failed</h4>
                                    <p className="text-xs text-rose-200/80 leading-relaxed break-words">{error}</p>
                                </div>
                            </div>
                        </div>
                    )}
                </div>

                {/* Program Info Overlay */}
                {abi && Array.isArray(abi.functions) && (
                    <div className="pt-4 border-t border-white/5">
                        <div className="flex items-center gap-2 mb-3">
                            <div className="h-px flex-1 bg-white/5" />
                            <span className="text-[10px] uppercase tracking-wider text-rose-pine-subtle font-medium">Program Interface</span>
                            <div className="h-px flex-1 bg-white/5" />
                        </div>
                        <div className="bg-rose-pine-surface/30 rounded-lg p-1 space-y-0.5">
                            {abi.functions.map((f: any) => (
                                <div key={f.name} className="flex items-center justify-between px-3 py-2 rounded-md hover:bg-white/5 transition-colors group">
                                    <span className="text-xs text-rose-pine-text family-mono group-hover:text-rose-pine-foam transition-colors">{f.name}</span>
                                    <span className="text-[10px] text-rose-pine-muted bg-black/20 px-1.5 py-0.5 rounded font-mono">
                                        fn::{f.index}
                                    </span>
                                </div>
                            ))}
                        </div>
                    </div>
                )}
            </div>
        </GlassCard>
    );
}
