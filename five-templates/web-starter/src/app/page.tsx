"use client";

import { useWallet } from "@solana/wallet-adapter-react";
import { useFive } from "@/components/providers/FiveProvider";
import { Navbar } from "@/components/layout/Navbar";
import { Button } from "@/components/ui/Button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/Card";
import { Activity, Code2, Cpu, Sparkles } from "lucide-react";

export default function Home() {
  const { connected, publicKey } = useWallet();
  const { isReady } = useFive();

  return (
    <div className="min-h-screen relative overflow-hidden flex flex-col">
      <Navbar />

      {/* Decorative background gradients */}
      <div className="absolute top-0 -left-1/4 w-1/2 h-1/2 bg-primary/20 rounded-full blur-[120px] pointer-events-none" />
      <div className="absolute bottom-0 -right-1/4 w-1/2 h-1/2 bg-accent/20 rounded-full blur-[120px] pointer-events-none" />

      <main className="flex-1 max-w-7xl mx-auto w-full px-6 pt-32 pb-16 relative z-10 flex flex-col items-center justify-center">
        <div className="text-center max-w-3xl mb-16 space-y-6">
          <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full glass border-primary/30 text-primary-foreground text-sm font-medium mb-4 animate-pulse">
            <Sparkles className="h-4 w-4" />
            <span>Introducing 5ive DApp Starter</span>
          </div>
          
          <h1 className="text-5xl md:text-7xl font-extrabold tracking-tight text-glow">
            Build Stunning <br />
            <span className="text-transparent bg-clip-text bg-gradient-to-r from-primary to-accent">
              Decentralized
            </span> Apps
          </h1>
          
          <p className="text-xl text-muted-foreground max-w-2xl mx-auto">
            The ultimate starting point for your Next.js and 5ive DSL projects. 
            Pre-configured with Solana Wallet Adapter, Glassmorphism UI, and TailwindCSS 4.
          </p>

          <div className="flex items-center justify-center gap-4 pt-4">
            <Button size="lg" className="gap-2">
              <Code2 className="h-5 w-5" />
              Start Building
            </Button>
            <Button size="lg" variant="glass" className="gap-2">
              Read Documentation
            </Button>
          </div>
        </div>

        <div className="grid md:grid-cols-2 gap-6 w-full max-w-4xl">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2 text-primary">
                <Activity className="h-5 w-5" />
                Wallet Status
              </CardTitle>
              <CardDescription>Solana wallet connection state</CardDescription>
            </CardHeader>
            <CardContent>
              {connected ? (
                <div className="space-y-4">
                  <div className="p-4 rounded-xl glass bg-green-500/10 border-green-500/20 text-green-400 font-mono text-sm break-all">
                    Connected: {publicKey?.toBase58()}
                  </div>
                </div>
              ) : (
                <div className="p-4 rounded-xl glass bg-yellow-500/10 border-yellow-500/20 text-yellow-500 text-sm">
                  Wallet is not connected. Please connect your wallet using the button in the navigation bar.
                </div>
              )}
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2 text-accent">
                <Cpu className="h-5 w-5" />
                5ive SDK Integration
              </CardTitle>
              <CardDescription>FiveProvider Context State</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                <div className={`p-4 rounded-xl glass border-white/10 font-mono text-sm ${isReady ? 'text-primary' : 'text-muted-foreground'}`}>
                  sdk_ready: {isReady ? "true" : "false"}
                </div>
                <Button 
                  variant="outline" 
                  className="w-full gap-2"
                  disabled={!connected || !isReady}
                  onClick={() => alert("Ready to execute 5ive scripts!")}
                >
                  <Cpu className="h-4 w-4" />
                  Test Execution
                </Button>
              </div>
            </CardContent>
          </Card>
        </div>
      </main>
    </div>
  );
}
