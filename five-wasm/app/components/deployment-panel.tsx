import React, { useState, useEffect } from 'react'
import { useWallet } from '@solana/wallet-adapter-react'
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui'
import { PublicKey } from '@solana/web3.js'
import { 
  DeploymentService, 
  DeploymentUI, 
  SolanaNetwork, 
  GasEstimation, 
  DeploymentProgress 
} from '../deployment-service'
import { WasmCompilerService } from '../wasm-compiler'

// UI Components
import { Button } from '../ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../ui/card'
import { Input } from '../ui/input'
import { Label } from '../ui/label'
import { Progress } from '../ui/progress'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../ui/select'
import { 
  AlertCircle, 
  CheckCircle2, 
  DollarSign, 
  ExternalLink, 
  Loader2, 
  Network, 
  Wallet,
  Zap 
} from 'lucide-react'

interface DeploymentPanelProps {
  bytecode?: Uint8Array
  onDeploymentComplete?: (result: any) => void
  className?: string
}

export function DeploymentPanel({ 
  bytecode, 
  onDeploymentComplete, 
  className 
}: DeploymentPanelProps) {
  // Wallet connection
  const { connected, publicKey, wallet } = useWallet()

  // Form state
  const [scriptName, setScriptName] = useState('')
  const [selectedNetwork, setSelectedNetwork] = useState<SolanaNetwork>('devnet')
  const [customRpcUrl, setCustomRpcUrl] = useState('')
  const [useCustomRpc, setUseCustomRpc] = useState(false)

  // Deployment state
  const [deploymentUI, setDeploymentUI] = useState<DeploymentUI | null>(null)
  const [gasEstimation, setGasEstimation] = useState<GasEstimation | null>(null)
  const [deploymentProgress, setDeploymentProgress] = useState<DeploymentProgress | null>(null)
  const [isEstimating, setIsEstimating] = useState(false)
  const [isDeploying, setIsDeploying] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)

  // Network connectivity
  const [networkConnected, setNetworkConnected] = useState<boolean | null>(null)

  // Initialize deployment UI
  useEffect(() => {
    const initializeDeploymentUI = async () => {
      try {
        const ui = new DeploymentUI()
        await ui.initialize()
        setDeploymentUI(ui)

        // Set up event listeners
        ui.on('deploymentProgress', (progress: DeploymentProgress) => {
          setDeploymentProgress(progress)
        })

        ui.on('deploymentSuccess', (result: any) => {
          setSuccess(`Deployment successful! Script deployed to ${result.scriptAddress?.toBase58()}`)
          setIsDeploying(false)
          onDeploymentComplete?.(result)
        })

        ui.on('deploymentError', (error: any) => {
          setError(error.error || 'Deployment failed')
          setIsDeploying(false)
        })

        ui.on('networkChanged', () => {
          checkNetworkConnectivity()
        })
      } catch (err) {
        setError(`Failed to initialize deployment service: ${err}`)
      }
    }

    initializeDeploymentUI()
  }, [onDeploymentComplete])

  // Update network when selection changes
  useEffect(() => {
    if (deploymentUI) {
      deploymentUI.setNetwork(selectedNetwork, useCustomRpc ? customRpcUrl : undefined)
      checkNetworkConnectivity()
    }
  }, [selectedNetwork, customRpcUrl, useCustomRpc, deploymentUI])

  // Check network connectivity
  const checkNetworkConnectivity = async () => {
    if (!deploymentUI) return
    
    try {
      const connected = await deploymentUI.checkConnectivity()
      setNetworkConnected(connected)
    } catch (err) {
      setNetworkConnected(false)
    }
  }

  // Estimate deployment costs
  const handleEstimate = async () => {
    if (!deploymentUI || !bytecode) {
      setError('No bytecode available for estimation')
      return
    }

    setIsEstimating(true)
    setError(null)

    try {
      const estimation = await deploymentUI.estimateDeployment(bytecode)
      setGasEstimation(estimation)
    } catch (err) {
      setError(`Estimation failed: ${err}`)
    } finally {
      setIsEstimating(false)
    }
  }

  // Deploy script
  const handleDeploy = async () => {
    if (!deploymentUI || !bytecode || !connected || !wallet) {
      setError('Missing requirements: bytecode, wallet connection, or deployment service')
      return
    }

    if (!scriptName.trim()) {
      setError('Script name is required')
      return
    }

    setIsDeploying(true)
    setError(null)
    setSuccess(null)

    try {
      const formData = {
        scriptName: scriptName.trim(),
        bytecode,
        network: selectedNetwork,
        customRpcUrl: useCustomRpc ? customRpcUrl : undefined,
      }

      await deploymentUI.deployScript(formData, wallet.adapter!)
    } catch (err) {
      setError(`Deployment failed: ${err}`)
      setIsDeploying(false)
    }
  }

  // Format SOL amount for display
  const formatSol = (amount: number) => {
    return `${amount.toFixed(6)} SOL`
  }

  // Get network display info
  const getNetworkDisplay = (network: SolanaNetwork) => {
    const displays = {
      'localnet': { name: 'Local Network', color: 'text-blue-600', icon: '🏠' },
      'devnet': { name: 'Devnet', color: 'text-green-600', icon: '🧪' },
      'testnet': { name: 'Testnet', color: 'text-yellow-600', icon: '🧪' },
      'mainnet-beta': { name: 'Mainnet Beta', color: 'text-red-600', icon: '🌐' }
    }
    return displays[network] || displays.devnet
  }

  const networkDisplay = getNetworkDisplay(selectedNetwork)

  return (
    <Card className={`glass-morphism border-none shadow-xl ${className}`}>
      <CardHeader className="pb-4">
        <CardTitle className="flex items-center gap-2 text-rose-100">
          <Zap className="h-5 w-5 text-rose-300" />
          Deploy to Solana
        </CardTitle>
        <CardDescription className="text-rose-200/70">
          Deploy your compiled Stacks bytecode to the Solana blockchain
        </CardDescription>
      </CardHeader>

      <CardContent className="space-y-6">
        {/* Network Selection */}
        <div className="space-y-3">
          <Label className="flex items-center gap-2 text-rose-100">
            <Network className="h-4 w-4" />
            Network
          </Label>
          <div className="flex gap-3 items-center">
            <Select value={selectedNetwork} onValueChange={(value: SolanaNetwork) => setSelectedNetwork(value)}>
              <SelectTrigger className="flex-1 bg-black/20 border-rose-300/20 text-rose-100">
                <SelectValue>
                  <div className="flex items-center gap-2">
                    <span>{networkDisplay.icon}</span>
                    <span className={networkDisplay.color}>{networkDisplay.name}</span>
                  </div>
                </SelectValue>
              </SelectTrigger>
              <SelectContent className="bg-black/90 border-rose-300/20">
                {DeploymentUI.NETWORKS.map((network) => {
                  const display = getNetworkDisplay(network.name)
                  return (
                    <SelectItem key={network.name} value={network.name} className="text-rose-100 focus:bg-rose-500/20">
                      <div className="flex items-center gap-2">
                        <span>{display.icon}</span>
                        <span>{display.name}</span>
                        {network.isMainnet && <span className="text-xs text-red-400">(Live)</span>}
                      </div>
                    </SelectItem>
                  )
                })}
              </SelectContent>
            </Select>
            
            {/* Network Status Indicator */}
            <div className="flex items-center gap-1">
              {networkConnected === null ? (
                <Loader2 className="h-4 w-4 animate-spin text-rose-300" />
              ) : networkConnected ? (
                <CheckCircle2 className="h-4 w-4 text-green-400" />
              ) : (
                <AlertCircle className="h-4 w-4 text-red-400" />
              )}
              <span className="text-xs text-rose-200/70">
                {networkConnected === null ? 'Checking...' : networkConnected ? 'Connected' : 'Disconnected'}
              </span>
            </div>
          </div>

          {/* Custom RPC URL */}
          <div className="space-y-2">
            <div className="flex items-center gap-2">
              <input
                type="checkbox"
                id="custom-rpc"
                checked={useCustomRpc}
                onChange={(e) => setUseCustomRpc(e.target.checked)}
                className="rounded border-rose-300/20"
              />
              <Label htmlFor="custom-rpc" className="text-sm text-rose-200/70">
                Use custom RPC URL
              </Label>
            </div>
            {useCustomRpc && (
              <Input
                placeholder="https://api.devnet.solana.com"
                value={customRpcUrl}
                onChange={(e) => setCustomRpcUrl(e.target.value)}
                className="bg-black/20 border-rose-300/20 text-rose-100"
              />
            )}
          </div>
        </div>

        {/* Wallet Connection */}
        <div className="space-y-3">
          <Label className="flex items-center gap-2 text-rose-100">
            <Wallet className="h-4 w-4" />
            Wallet Connection
          </Label>
          <div className="flex items-center gap-3">
            <WalletMultiButton className="!bg-gradient-to-r !from-rose-600 !to-pink-600 !text-white !border-none !rounded-md !h-10" />
            {connected && publicKey && (
              <div className="text-sm text-rose-200/70">
                {publicKey.toBase58().slice(0, 8)}...{publicKey.toBase58().slice(-8)}
              </div>
            )}
          </div>
        </div>

        {/* Script Configuration */}
        <div className="space-y-3">
          <Label htmlFor="script-name" className="text-rose-100">
            Script Name
          </Label>
          <Input
            id="script-name"
            placeholder="my-stacks-script"
            value={scriptName}
            onChange={(e) => setScriptName(e.target.value)}
            className="bg-black/20 border-rose-300/20 text-rose-100"
            disabled={isDeploying}
          />
        </div>

        {/* Bytecode Info */}
        {bytecode && (
          <div className="space-y-3">
            <Label className="text-rose-100">Bytecode</Label>
            <div className="bg-black/20 border border-rose-300/20 rounded-md p-3">
              <div className="text-sm text-rose-200/70">
                Size: {bytecode.length} bytes
              </div>
              <div className="text-xs text-rose-200/50 font-mono mt-1">
                {Array.from(bytecode.slice(0, 16))
                  .map(b => b.toString(16).padStart(2, '0'))
                  .join(' ')}
                {bytecode.length > 16 && '...'}
              </div>
            </div>
          </div>
        )}

        {/* Cost Estimation */}
        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <Label className="flex items-center gap-2 text-rose-100">
              <DollarSign className="h-4 w-4" />
              Cost Estimation
            </Label>
            <Button
              variant="outline"
              size="sm"
              onClick={handleEstimate}
              disabled={!bytecode || isEstimating}
              className="bg-black/20 border-rose-300/20 text-rose-100 hover:bg-rose-500/20"
            >
              {isEstimating ? (
                <>
                  <Loader2 className="h-3 w-3 animate-spin mr-1" />
                  Estimating...
                </>
              ) : (
                'Estimate'
              )}
            </Button>
          </div>

          {gasEstimation && (
            <div className="bg-black/20 border border-rose-300/20 rounded-md p-4 space-y-2">
              <div className="grid grid-cols-2 gap-4 text-sm">
                <div>
                  <span className="text-rose-200/70">Compute Units:</span>
                  <span className="ml-2 text-rose-100">{gasEstimation.computeUnits.toLocaleString()}</span>
                </div>
                <div>
                  <span className="text-rose-200/70">Transaction Fee:</span>
                  <span className="ml-2 text-rose-100">{formatSol(gasEstimation.transactionFee)}</span>
                </div>
                <div>
                  <span className="text-rose-200/70">Rent Exempt:</span>
                  <span className="ml-2 text-rose-100">{formatSol(gasEstimation.rentExemptBalance)}</span>
                </div>
                <div className="font-medium">
                  <span className="text-rose-200/70">Total Cost:</span>
                  <span className="ml-2 text-rose-100">{formatSol(gasEstimation.totalCost)}</span>
                </div>
              </div>
            </div>
          )}
        </div>

        {/* Deployment Progress */}
        {deploymentProgress && (
          <div className="space-y-3">
            <Label className="text-rose-100">Deployment Progress</Label>
            <div className="bg-black/20 border border-rose-300/20 rounded-md p-4 space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-sm text-rose-100">{deploymentProgress.description}</span>
                <span className="text-sm text-rose-200/70">{deploymentProgress.progress}%</span>
              </div>
              <Progress value={deploymentProgress.progress} className="h-2" />
              {deploymentProgress.signature && (
                <div className="flex items-center gap-2 text-xs">
                  <span className="text-rose-200/70">Transaction:</span>
                  <a
                    href={`https://explorer.solana.com/tx/${deploymentProgress.signature}${selectedNetwork !== 'mainnet-beta' ? `?cluster=${selectedNetwork}` : ''}`}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-rose-300 hover:text-rose-200 flex items-center gap-1"
                  >
                    {deploymentProgress.signature.slice(0, 8)}...{deploymentProgress.signature.slice(-8)}
                    <ExternalLink className="h-3 w-3" />
                  </a>
                </div>
              )}
            </div>
          </div>
        )}

        {/* Error Display */}
        {error && (
          <div className="bg-red-900/20 border border-red-500/30 rounded-md p-3">
            <div className="flex items-center gap-2">
              <AlertCircle className="h-4 w-4 text-red-400" />
              <span className="text-sm text-red-200">{error}</span>
            </div>
          </div>
        )}

        {/* Success Display */}
        {success && (
          <div className="bg-green-900/20 border border-green-500/30 rounded-md p-3">
            <div className="flex items-center gap-2">
              <CheckCircle2 className="h-4 w-4 text-green-400" />
              <span className="text-sm text-green-200">{success}</span>
            </div>
          </div>
        )}

        {/* Deploy Button */}
        <Button
          onClick={handleDeploy}
          disabled={!bytecode || !connected || !scriptName.trim() || isDeploying || networkConnected === false}
          className="w-full bg-gradient-to-r from-rose-600 to-pink-600 hover:from-rose-700 hover:to-pink-700 text-white border-none"
        >
          {isDeploying ? (
            <>
              <Loader2 className="h-4 w-4 animate-spin mr-2" />
              Deploying...
            </>
          ) : (
            <>
              <Zap className="h-4 w-4 mr-2" />
              Deploy Script
            </>
          )}
        </Button>

        {/* Requirements checklist */}
        <div className="text-xs text-rose-200/50 space-y-1">
          <div>Requirements:</div>
          <div className={`flex items-center gap-1 ${bytecode ? 'text-green-400' : 'text-rose-300'}`}>
            {bytecode ? '✓' : '○'} Compiled bytecode
          </div>
          <div className={`flex items-center gap-1 ${connected ? 'text-green-400' : 'text-rose-300'}`}>
            {connected ? '✓' : '○'} Wallet connected
          </div>
          <div className={`flex items-center gap-1 ${scriptName.trim() ? 'text-green-400' : 'text-rose-300'}`}>
            {scriptName.trim() ? '✓' : '○'} Script name provided
          </div>
          <div className={`flex items-center gap-1 ${networkConnected ? 'text-green-400' : 'text-rose-300'}`}>
            {networkConnected ? '✓' : '○'} Network connectivity
          </div>
        </div>
      </CardContent>
    </Card>
  )
}