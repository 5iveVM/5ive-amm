import React, { useState, useEffect, useMemo } from 'react'
import { ConnectionProvider, WalletProvider } from '@solana/wallet-adapter-react'
import { WalletAdapterNetwork } from '@solana/wallet-adapter-base'
import { WalletModalProvider } from '@solana/wallet-adapter-react-ui'
import {
  PhantomWalletAdapter,
  SolflareWalletAdapter,
  SolletWalletAdapter,
  SolletExtensionWalletAdapter,
} from '@solana/wallet-adapter-wallets'
import { clusterApiUrl } from '@solana/web3.js'

// Internal components
import { DeploymentPanel } from './deployment-panel'
import { DeploymentHistory } from './deployment-history'
import { Toaster } from './toaster'
import { useToast } from '../hooks/use-toast'
import { WasmCompilerService } from '../wasm-compiler'

// UI Components
import { Button } from '../ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../ui/card'
import { Input } from '../ui/input'
import { Label } from '../ui/label'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../ui/tabs'
import { 
  Code, 
  FileCode, 
  History, 
  Play, 
  Rocket,
  Upload
} from 'lucide-react'

// Wallet Adapter CSS (you may need to include this in your global CSS)
import '@solana/wallet-adapter-react-ui/styles.css'

interface DeploymentPlaygroundProps {
  className?: string
}

export function DeploymentPlayground({ className }: DeploymentPlaygroundProps) {
  // Wallet configuration
  const network = WalletAdapterNetwork.Devnet
  const endpoint = useMemo(() => clusterApiUrl(network), [network])
  
  const wallets = useMemo(
    () => [
      new PhantomWalletAdapter(),
      new SolflareWalletAdapter(),
      new SolletWalletAdapter(),
      new SolletExtensionWalletAdapter(),
    ],
    []
  )

  // Component state
  const [sourceCode, setSourceCode] = useState('')
  const [bytecode, setBytecode] = useState<Uint8Array | null>(null)
  const [isCompiling, setIsCompiling] = useState(false)
  const [wasmCompiler, setWasmCompiler] = useState<WasmCompilerService | null>(null)
  const [activeTab, setActiveTab] = useState('compile')
  
  const { toast } = useToast()

  // Initialize WASM compiler
  useEffect(() => {
    const initCompiler = async () => {
      try {
        const compiler = new WasmCompilerService()
        await compiler.initialize()
        setWasmCompiler(compiler)
        
        toast({
          title: "Compiler Ready",
          description: "WASM compiler initialized successfully",
        })
      } catch (error) {
        toast({
          variant: "destructive",
          title: "Compiler Error",
          description: `Failed to initialize WASM compiler: ${error}`,
        })
      }
    }

    initCompiler()
  }, [toast])

  // Sample Stacks code for demonstration
  const sampleCode = `// Simple vault contract
contract SimpleVault {
    field owner: pubkey
    field balance: u64
    
    instruction init(owner: pubkey) {
        self.owner = owner
        self.balance = 0
    }
    
    instruction deposit(amount: u64) {
        require(amount > 0, "Amount must be positive")
        self.balance = self.balance + amount
    }
    
    instruction withdraw(amount: u64) {
        require(self.balance >= amount, "Insufficient balance")
        self.balance = self.balance - amount
    }
    
    view get_balance() -> u64 {
        return self.balance
    }
}`

  const handleCompile = async () => {
    if (!wasmCompiler) {
      toast({
        variant: "destructive",
        title: "Compiler Not Ready",
        description: "WASM compiler is not initialized yet",
      })
      return
    }

    if (!sourceCode.trim()) {
      toast({
        variant: "destructive",
        title: "No Source Code",
        description: "Please enter some Stacks source code to compile",
      })
      return
    }

    setIsCompiling(true)
    try {
      const result = await wasmCompiler.compileToStacksBytecode(sourceCode)
      
      if (result.success && result.bytecode) {
        setBytecode(result.bytecode)
        setActiveTab('deploy')
        
        toast({
          title: "Compilation Successful",
          description: `Generated ${result.bytecode.length} bytes of bytecode`,
        })
      } else {
        toast({
          variant: "destructive",
          title: "Compilation Failed",
          description: result.error || "Unknown compilation error",
        })
      }
    } catch (error) {
      toast({
        variant: "destructive",
        title: "Compilation Error",
        description: `${error}`,
      })
    } finally {
      setIsCompiling(false)
    }
  }

  const handleLoadSample = () => {
    setSourceCode(sampleCode)
    toast({
      title: "Sample Loaded",
      description: "Sample vault contract loaded into editor",
    })
  }

  const handleFileUpload = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0]
    if (file) {
      const reader = new FileReader()
      reader.onload = (e) => {
        const content = e.target?.result as string
        setSourceCode(content)
        toast({
          title: "File Loaded",
          description: `Loaded ${file.name}`,
        })
      }
      reader.readAsText(file)
    }
  }

  const handleDeploymentComplete = (result: any) => {
    if (result.success) {
      setActiveTab('history')
      toast({
        title: "Deployment Successful",
        description: `Script deployed to ${result.scriptAddress?.toBase58()}`,
      })
    }
  }

  return (
    <ConnectionProvider endpoint={endpoint}>
      <WalletProvider wallets={wallets} autoConnect>
        <WalletModalProvider>
          <div className={`deployment-playground min-h-screen bg-gradient-to-br from-rose-950 via-black to-pink-950 ${className}`}>
            <div className="container mx-auto p-6 space-y-6">
              {/* Header */}
              <div className="text-center space-y-2">
                <h1 className="text-4xl font-bold text-rose-100 flex items-center justify-center gap-3">
                  <Rocket className="h-8 w-8 text-rose-300" />
                  Stacks VM Deployment Playground
                </h1>
                <p className="text-rose-200/70 text-lg">
                  Compile and deploy Stacks bytecode to Solana
                </p>
              </div>

              {/* Main Content */}
              <Tabs value={activeTab} onValueChange={setActiveTab} className="w-full">
                <TabsList className="grid w-full grid-cols-3 bg-black/20 border border-rose-300/20">
                  <TabsTrigger 
                    value="compile" 
                    className="data-[state=active]:bg-rose-600 data-[state=active]:text-white text-rose-200"
                  >
                    <Code className="h-4 w-4 mr-2" />
                    Compile
                  </TabsTrigger>
                  <TabsTrigger 
                    value="deploy" 
                    className="data-[state=active]:bg-rose-600 data-[state=active]:text-white text-rose-200"
                    disabled={!bytecode}
                  >
                    <Rocket className="h-4 w-4 mr-2" />
                    Deploy
                  </TabsTrigger>
                  <TabsTrigger 
                    value="history" 
                    className="data-[state=active]:bg-rose-600 data-[state=active]:text-white text-rose-200"
                  >
                    <History className="h-4 w-4 mr-2" />
                    History
                  </TabsTrigger>
                </TabsList>

                {/* Compile Tab */}
                <TabsContent value="compile" className="space-y-6">
                  <Card className="glass-morphism border-none shadow-xl">
                    <CardHeader>
                      <CardTitle className="flex items-center gap-2 text-rose-100">
                        <FileCode className="h-5 w-5 text-rose-300" />
                        Stacks Source Code
                      </CardTitle>
                      <CardDescription className="text-rose-200/70">
                        Write or upload your Stacks smart contract code
                      </CardDescription>
                    </CardHeader>
                    <CardContent className="space-y-4">
                      {/* File upload and sample loading */}
                      <div className="flex gap-3">
                        <Button
                          variant="outline"
                          onClick={handleLoadSample}
                          className="bg-black/20 border-rose-300/20 text-rose-100 hover:bg-rose-500/20"
                        >
                          <FileCode className="h-4 w-4 mr-2" />
                          Load Sample
                        </Button>
                        <Label 
                          htmlFor="file-upload" 
                          className="inline-flex items-center gap-2 px-4 py-2 rounded-md border border-rose-300/20 bg-black/20 text-rose-100 hover:bg-rose-500/20 cursor-pointer"
                        >
                          <Upload className="h-4 w-4" />
                          Upload File
                        </Label>
                        <Input
                          id="file-upload"
                          type="file"
                          accept=".stacks,.txt"
                          onChange={handleFileUpload}
                          className="hidden"
                        />
                      </div>

                      {/* Code editor */}
                      <div className="space-y-2">
                        <Label htmlFor="source-code" className="text-rose-100">
                          Source Code
                        </Label>
                        <textarea
                          id="source-code"
                          value={sourceCode}
                          onChange={(e) => setSourceCode(e.target.value)}
                          placeholder="Enter your Stacks source code here..."
                          className="w-full h-64 px-4 py-3 bg-black/40 border border-rose-300/20 rounded-md text-rose-100 font-mono text-sm resize-none focus:outline-none focus:ring-2 focus:ring-rose-500 focus:border-transparent"
                        />
                      </div>

                      {/* Compile button */}
                      <Button
                        onClick={handleCompile}
                        disabled={!wasmCompiler || !sourceCode.trim() || isCompiling}
                        className="w-full bg-gradient-to-r from-rose-600 to-pink-600 hover:from-rose-700 hover:to-pink-700 text-white border-none"
                      >
                        {isCompiling ? (
                          <>
                            <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2" />
                            Compiling...
                          </>
                        ) : (
                          <>
                            <Play className="h-4 w-4 mr-2" />
                            Compile to Bytecode
                          </>
                        )}
                      </Button>

                      {/* Bytecode display */}
                      {bytecode && (
                        <div className="space-y-2">
                          <Label className="text-rose-100">Generated Bytecode</Label>
                          <div className="bg-black/40 border border-rose-300/20 rounded-md p-4">
                            <div className="text-sm text-rose-200/70 mb-2">
                              Size: {bytecode.length} bytes
                            </div>
                            <div className="text-xs text-rose-200/50 font-mono break-all">
                              {Array.from(bytecode.slice(0, 64))
                                .map(b => b.toString(16).padStart(2, '0'))
                                .join(' ')}
                              {bytecode.length > 64 && '...'}
                            </div>
                          </div>
                        </div>
                      )}
                    </CardContent>
                  </Card>
                </TabsContent>

                {/* Deploy Tab */}
                <TabsContent value="deploy" className="space-y-6">
                  <DeploymentPanel
                    bytecode={bytecode || undefined}
                    onDeploymentComplete={handleDeploymentComplete}
                  />
                </TabsContent>

                {/* History Tab */}
                <TabsContent value="history" className="space-y-6">
                  <DeploymentHistory
                    onRedeploy={async (deploymentId) => {
                      toast({
                        title: "Redeploy",
                        description: "Redeploy functionality requires original bytecode",
                      })
                    }}
                    onClearHistory={async () => {
                      toast({
                        title: "History Cleared",
                        description: "Deployment history has been cleared",
                      })
                    }}
                  />
                </TabsContent>
              </Tabs>
            </div>

            {/* Toast notifications */}
            <Toaster />
          </div>
        </WalletModalProvider>
      </WalletProvider>
    </ConnectionProvider>
  )
}

// CSS for glass morphism and Rose Pine Moon theme
export const playgroundStyles = `
.glass-morphism {
  background: rgba(255, 255, 255, 0.03);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.deployment-playground {
  background: linear-gradient(135deg, #1f1d2e 0%, #000000 50%, #1f1d2e 100%);
  min-height: 100vh;
}

/* Wallet button styling */
.wallet-adapter-button {
  background: linear-gradient(135deg, #e11d48 0%, #ec4899 100%) !important;
  border: none !important;
  border-radius: 6px !important;
  color: white !important;
  font-weight: 500 !important;
}

.wallet-adapter-button:hover {
  background: linear-gradient(135deg, #be185d 0%, #db2777 100%) !important;
}

/* Toast styling for Rose Pine Moon theme */
.toast {
  background: rgba(0, 0, 0, 0.9) !important;
  border: 1px solid rgba(244, 204, 221, 0.2) !important;
  color: rgb(244 204 221) !important;
}

.toast[data-variant="destructive"] {
  background: rgba(127, 29, 29, 0.9) !important;
  border: 1px solid rgba(239, 68, 68, 0.3) !important;
  color: rgb(254 202 202) !important;
}
`