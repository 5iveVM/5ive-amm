/**
 * Deployment UI example using real Solana deployments only.
 */

import React from 'react'
import ReactDOM from 'react-dom/client'
import { DeploymentPlayground } from '../app/components/deployment-playground'

// Import styles
import '../app/styles/deployment-ui.css'
import '@solana/wallet-adapter-react-ui/styles.css'

/**
 * Main App Component
 * 
 * Demonstrates the complete deployment playground with:
 * - Code compilation with WASM compiler
 * - Real Solana network deployment
 * - Deployment history tracking
 * - Wallet integration
 * - Toast notifications
 */
function App() {
  return (
    <div className="min-h-screen">
      <DeploymentPlayground />
    </div>
  )
}

/**
 * Example of using individual components
 */
export function IndividualComponentsExample() {
  const [bytecode, setBytecode] = React.useState<Uint8Array | null>(null)

  // Example bytecode (in real usage, this would come from the compiler)
  React.useEffect(() => {
    // Simulate compiled bytecode
    const exampleBytecode = new Uint8Array([
      0x35, 0x49, 0x56, 0x45, // Magic bytes "5IVE"
      0x00, 0x00, 0x00, 0x00, // Features (u32)
      0x00, 0x00, // public/total function counts
      0x10, 0x20, 0x30, 0x40, // Some opcodes
      0x00, // HALT
    ])
    setBytecode(exampleBytecode)
  }, [])

  const handleDeploymentComplete = (result: any) => {
    console.log('Deployment completed:', result)
    if (result.success) {
      alert(`Successfully deployed to: ${result.scriptAddress?.toBase58()}`)
    } else {
      alert(`Deployment failed: ${result.error}`)
    }
  }

  const handleRedeploy = async (deploymentId: string) => {
    console.log('Redeploying:', deploymentId)
    // In a real app, you would retrieve the original bytecode and redeploy
    alert('Redeploy requires original source code compilation')
  }

  const handleClearHistory = async () => {
    console.log('Clearing deployment history')
    // History is cleared from localStorage
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-rose-950 via-black to-pink-950 p-6">
      <div className="container mx-auto space-y-6">
        <h1 className="text-3xl font-bold text-rose-100 text-center mb-8">
          Individual Components Example
        </h1>
        
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          {/* Deployment Panel */}
          <div>
            <h2 className="text-xl font-semibold text-rose-100 mb-4">Deployment Panel</h2>
            {/* Note: You would need to wrap this in wallet providers in a real app */}
            {/*
            <DeploymentPanel
              bytecode={bytecode || undefined}
              onDeploymentComplete={handleDeploymentComplete}
            />
            */}
            <div className="glass-morphism p-6 rounded-lg">
              <p className="text-rose-200/70">
                DeploymentPanel component would render here.
                Wrap in ConnectionProvider and WalletProvider to use.
              </p>
            </div>
          </div>

          {/* Deployment History */}
          <div>
            <h2 className="text-xl font-semibold text-rose-100 mb-4">Deployment History</h2>
            {/*
            <DeploymentHistory
              onRedeploy={handleRedeploy}
              onClearHistory={handleClearHistory}
            />
            */}
            <div className="glass-morphism p-6 rounded-lg">
              <p className="text-rose-200/70">
                DeploymentHistory component would render here.
                Shows past deployments from localStorage.
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

/**
 * Usage Instructions
 * 
 * To use these components in your React app:
 * 
 * 1. Install dependencies:
 *    npm install react react-dom @solana/web3.js @solana/wallet-adapter-react
 *    npm install @solana/wallet-adapter-react-ui @solana/wallet-adapter-wallets
 *    npm install @radix-ui/react-select @radix-ui/react-progress @radix-ui/react-toast
 *    npm install lucide-react clsx tailwind-merge class-variance-authority
 * 
 * 2. Set up Tailwind CSS with the Rose Pine Moon theme
 * 
 * 3. Import and use the components:
 * 
 *    import { DeploymentPlayground } from './app/components/deployment-playground'
 *    import './app/styles/deployment-ui.css'
 *    import '@solana/wallet-adapter-react-ui/styles.css'
 * 
 * 4. For individual components, wrap in wallet providers:
 * 
 *    <ConnectionProvider endpoint={endpoint}>
 *      <WalletProvider wallets={wallets} autoConnect>
 *        <WalletModalProvider>
 *          <DeploymentPanel bytecode={bytecode} onDeploymentComplete={handleComplete} />
 *          <DeploymentHistory onRedeploy={handleRedeploy} onClearHistory={handleClear} />
 *        </WalletModalProvider>
 *      </WalletProvider>
 *    </ConnectionProvider>
 * 
 * 5. Ensure you have a working WASM compiler service for bytecode generation
 */

// Export for use in other files
export { App, DeploymentPlayground }

// Render if this file is run directly
if (typeof window !== 'undefined' && document.getElementById('root')) {
  const root = ReactDOM.createRoot(document.getElementById('root')!)
  root.render(<App />)
}
