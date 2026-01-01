import React, { useState, useEffect } from 'react'
import { ColumnDef } from '@tanstack/react-table'
import { 
  DeploymentHistoryEntry, 
  DeploymentUtils, 
  SolanaNetwork 
} from '../deployment-service'

// UI Components
import { Button } from '../ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../ui/card'
import { DataTable } from '../ui/data-table'
import { 
  Select, 
  SelectContent, 
  SelectItem, 
  SelectTrigger, 
  SelectValue 
} from '../ui/select'
import { 
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '../ui/dropdown-menu'
import { 
  AlertCircle,
  Calendar,
  CheckCircle2,
  ExternalLink,
  History,
  MoreHorizontal,
  RefreshCw,
  Trash2,
  XCircle
} from 'lucide-react'

interface DeploymentHistoryProps {
  onRedeploy?: (deploymentId: string) => Promise<void>
  onClearHistory?: () => Promise<void>
  className?: string
}

export function DeploymentHistory({ 
  onRedeploy, 
  onClearHistory, 
  className 
}: DeploymentHistoryProps) {
  const [deployments, setDeployments] = useState<DeploymentHistoryEntry[]>([])
  const [filteredDeployments, setFilteredDeployments] = useState<DeploymentHistoryEntry[]>([])
  const [networkFilter, setNetworkFilter] = useState<SolanaNetwork | 'all'>('all')
  const [statusFilter, setStatusFilter] = useState<'all' | 'success' | 'failed'>('all')
  const [isLoading, setIsLoading] = useState(true)

  // Load deployment history from localStorage
  useEffect(() => {
    loadDeploymentHistory()
  }, [])

  // Apply filters
  useEffect(() => {
    let filtered = [...deployments]

    if (networkFilter !== 'all') {
      filtered = filtered.filter(deployment => deployment.result.network === networkFilter)
    }

    if (statusFilter !== 'all') {
      filtered = filtered.filter(deployment => 
        statusFilter === 'success' ? deployment.result.success : !deployment.result.success
      )
    }

    // Sort by deployment date (newest first)
    filtered.sort((a, b) => 
      new Date(b.result.deployedAt).getTime() - new Date(a.result.deployedAt).getTime()
    )

    setFilteredDeployments(filtered)
  }, [deployments, networkFilter, statusFilter])

  const loadDeploymentHistory = () => {
    setIsLoading(true)
    try {
      const stored = localStorage.getItem('stacks_deployment_history')
      if (stored) {
        const history: DeploymentHistoryEntry[] = JSON.parse(stored)
        setDeployments(history)
      }
    } catch (error) {
      console.error('Failed to load deployment history:', error)
    } finally {
      setIsLoading(false)
    }
  }

  const handleClearHistory = async () => {
    if (window.confirm('Are you sure you want to clear all deployment history? This cannot be undone.')) {
      localStorage.removeItem('stacks_deployment_history')
      setDeployments([])
      onClearHistory?.()
    }
  }

  const handleRedeploy = async (deploymentId: string) => {
    if (onRedeploy) {
      try {
        await onRedeploy(deploymentId)
        // Reload history after successful redeploy
        loadDeploymentHistory()
      } catch (error) {
        console.error('Redeploy failed:', error)
      }
    }
  }

  const formatDate = (date: string | Date) => {
    return new Date(date).toLocaleString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    })
  }

  const formatSol = (amount: number) => {
    return `${amount.toFixed(6)} SOL`
  }

  const getStatusIcon = (success: boolean) => {
    return success ? (
      <CheckCircle2 className="h-4 w-4 text-green-400" />
    ) : (
      <XCircle className="h-4 w-4 text-red-400" />
    )
  }

  const getNetworkDisplay = (network: SolanaNetwork) => {
    const displays = {
      'localnet': { name: 'Local', color: 'text-blue-400', icon: '🏠' },
      'devnet': { name: 'Devnet', color: 'text-green-400', icon: '🧪' },
      'testnet': { name: 'Testnet', color: 'text-yellow-400', icon: '🧪' },
      'mainnet-beta': { name: 'Mainnet', color: 'text-red-400', icon: '🌐' }
    }
    return displays[network] || displays.devnet
  }

  // Define table columns
  const columns: ColumnDef<DeploymentHistoryEntry>[] = [
    {
      accessorKey: "name",
      header: "Script Name",
      cell: ({ row }) => {
        const deployment = row.original
        return (
          <div className="font-medium text-rose-100">
            {deployment.name}
          </div>
        )
      },
    },
    {
      accessorKey: "result.network",
      header: "Network",
      cell: ({ row }) => {
        const network = row.getValue("result.network") as SolanaNetwork
        const display = getNetworkDisplay(network)
        return (
          <div className={`flex items-center gap-1 ${display.color}`}>
            <span>{display.icon}</span>
            <span className="text-sm">{display.name}</span>
          </div>
        )
      },
    },
    {
      accessorKey: "result.success",
      header: "Status",
      cell: ({ row }) => {
        const success = row.getValue("result.success") as boolean
        const deployment = row.original
        return (
          <div className="flex items-center gap-2">
            {getStatusIcon(success)}
            <span className={`text-sm ${success ? 'text-green-400' : 'text-red-400'}`}>
              {success ? 'Success' : 'Failed'}
            </span>
            {!success && deployment.result.error && (
              <div className="text-xs text-rose-200/50 max-w-[200px] truncate">
                {deployment.result.error}
              </div>
            )}
          </div>
        )
      },
    },
    {
      accessorKey: "result.deployedAt",
      header: "Deployed At",
      cell: ({ row }) => {
        const date = row.getValue("result.deployedAt") as string
        return (
          <div className="text-sm text-rose-200/70">
            {formatDate(date)}
          </div>
        )
      },
    },
    {
      accessorKey: "result.cost",
      header: "Cost",
      cell: ({ row }) => {
        const cost = row.getValue("result.cost") as number
        return (
          <div className="text-sm text-rose-200/70">
            {formatSol(cost)}
          </div>
        )
      },
    },
    {
      accessorKey: "result.scriptAddress",
      header: "Script Address",
      cell: ({ row }) => {
        const deployment = row.original
        const address = deployment.result.scriptAddress
        const network = deployment.result.network
        
        if (!address || !deployment.result.success) {
          return <span className="text-rose-200/50">-</span>
        }

        const addressStr = typeof address === 'string' ? address : address.toBase58()
        const explorerUrl = DeploymentUtils.getAccountExplorerUrl(addressStr, network)
        
        return (
          <div className="flex items-center gap-2">
            <span className="text-sm font-mono text-rose-200/70">
              {addressStr.slice(0, 8)}...{addressStr.slice(-8)}
            </span>
            <a
              href={explorerUrl}
              target="_blank"
              rel="noopener noreferrer"
              className="text-rose-300 hover:text-rose-200"
            >
              <ExternalLink className="h-3 w-3" />
            </a>
          </div>
        )
      },
    },
    {
      id: "actions",
      cell: ({ row }) => {
        const deployment = row.original
        
        return (
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button 
                variant="ghost" 
                className="h-8 w-8 p-0 text-rose-200 hover:bg-rose-500/20"
              >
                <MoreHorizontal className="h-4 w-4" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent 
              align="end" 
              className="bg-black/90 border-rose-300/20 text-rose-100"
            >
              {deployment.result.success && deployment.result.signature && (
                <DropdownMenuItem 
                  className="focus:bg-rose-500/20"
                  onClick={() => {
                    const explorerUrl = DeploymentUtils.getExplorerUrl(
                      deployment.result.signature!, 
                      deployment.result.network
                    )
                    window.open(explorerUrl, '_blank')
                  }}
                >
                  <ExternalLink className="mr-2 h-4 w-4" />
                  View Transaction
                </DropdownMenuItem>
              )}
              <DropdownMenuItem 
                className="focus:bg-rose-500/20"
                onClick={() => handleRedeploy(deployment.id)}
                disabled={!onRedeploy}
              >
                <RefreshCw className="mr-2 h-4 w-4" />
                Redeploy
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        )
      },
    },
  ]

  return (
    <Card className={`glass-morphism border-none shadow-xl ${className}`}>
      <CardHeader className="pb-4">
        <div className="flex items-center justify-between">
          <div>
            <CardTitle className="flex items-center gap-2 text-rose-100">
              <History className="h-5 w-5 text-rose-300" />
              Deployment History
            </CardTitle>
            <CardDescription className="text-rose-200/70">
              View and manage your previous deployments
            </CardDescription>
          </div>
          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={loadDeploymentHistory}
              className="bg-black/20 border-rose-300/20 text-rose-100 hover:bg-rose-500/20"
            >
              <RefreshCw className="h-3 w-3 mr-1" />
              Refresh
            </Button>
            {deployments.length > 0 && (
              <Button
                variant="outline"
                size="sm"
                onClick={handleClearHistory}
                className="bg-black/20 border-rose-300/20 text-red-300 hover:bg-red-500/20"
              >
                <Trash2 className="h-3 w-3 mr-1" />
                Clear
              </Button>
            )}
          </div>
        </div>
      </CardHeader>

      <CardContent>
        {/* Filters */}
        <div className="flex gap-4 mb-6">
          <div className="flex items-center gap-2">
            <label className="text-sm text-rose-200/70">Network:</label>
            <Select value={networkFilter} onValueChange={(value) => setNetworkFilter(value as any)}>
              <SelectTrigger className="w-32 bg-black/20 border-rose-300/20 text-rose-100">
                <SelectValue />
              </SelectTrigger>
              <SelectContent className="bg-black/90 border-rose-300/20">
                <SelectItem value="all" className="text-rose-100 focus:bg-rose-500/20">All</SelectItem>
                <SelectItem value="localnet" className="text-rose-100 focus:bg-rose-500/20">Local</SelectItem>
                <SelectItem value="devnet" className="text-rose-100 focus:bg-rose-500/20">Devnet</SelectItem>
                <SelectItem value="testnet" className="text-rose-100 focus:bg-rose-500/20">Testnet</SelectItem>
                <SelectItem value="mainnet-beta" className="text-rose-100 focus:bg-rose-500/20">Mainnet</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div className="flex items-center gap-2">
            <label className="text-sm text-rose-200/70">Status:</label>
            <Select value={statusFilter} onValueChange={(value) => setStatusFilter(value as any)}>
              <SelectTrigger className="w-32 bg-black/20 border-rose-300/20 text-rose-100">
                <SelectValue />
              </SelectTrigger>
              <SelectContent className="bg-black/90 border-rose-300/20">
                <SelectItem value="all" className="text-rose-100 focus:bg-rose-500/20">All</SelectItem>
                <SelectItem value="success" className="text-rose-100 focus:bg-rose-500/20">Success</SelectItem>
                <SelectItem value="failed" className="text-rose-100 focus:bg-rose-500/20">Failed</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>

        {/* Data Table */}
        {isLoading ? (
          <div className="flex justify-center py-8">
            <div className="text-rose-200/70">Loading deployment history...</div>
          </div>
        ) : filteredDeployments.length === 0 ? (
          <div className="text-center py-8">
            <Calendar className="h-12 w-12 text-rose-300/50 mx-auto mb-4" />
            <h3 className="text-lg font-medium text-rose-100 mb-2">No deployments found</h3>
            <p className="text-rose-200/70">
              {deployments.length === 0 
                ? "Deploy your first script to see it here"
                : "No deployments match the selected filters"
              }
            </p>
          </div>
        ) : (
          <div className="deployment-history-table">
            <DataTable
              columns={columns}
              data={filteredDeployments}
              searchKey="name"
              searchPlaceholder="Search scripts..."
            />
          </div>
        )}

        {/* Summary Stats */}
        {deployments.length > 0 && (
          <div className="mt-6 grid grid-cols-3 gap-4 text-center">
            <div className="bg-black/20 rounded-md p-3">
              <div className="text-2xl font-bold text-rose-100">{deployments.length}</div>
              <div className="text-sm text-rose-200/70">Total Deployments</div>
            </div>
            <div className="bg-black/20 rounded-md p-3">
              <div className="text-2xl font-bold text-green-400">
                {deployments.filter(d => d.result.success).length}
              </div>
              <div className="text-sm text-rose-200/70">Successful</div>
            </div>
            <div className="bg-black/20 rounded-md p-3">
              <div className="text-2xl font-bold text-rose-100">
                {formatSol(deployments.reduce((sum, d) => sum + (d.result.cost || 0), 0))}
              </div>
              <div className="text-sm text-rose-200/70">Total Cost</div>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  )
}

// CSS for glass morphism styling
export const deploymentHistoryStyles = `
.glass-morphism {
  background: rgba(255, 255, 255, 0.03);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.deployment-history-table .data-table {
  background: rgba(0, 0, 0, 0.2);
  border-radius: 8px;
}

.deployment-history-table table {
  background: transparent;
}

.deployment-history-table th {
  background: rgba(0, 0, 0, 0.3);
  color: rgb(244 204 221);
  border-bottom: 1px solid rgba(244 204 221, 0.2);
}

.deployment-history-table td {
  border-bottom: 1px solid rgba(244 204 221, 0.1);
}

.deployment-history-table tr:hover {
  background: rgba(244 63 94, 0.1);
}
`