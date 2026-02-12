"use client";

import { useIdeStore } from "@/stores/ide-store";
import { useEffect, useState } from "react";
import { ChevronDown, ChevronRight, FileCode, Function, Layers, Circle, Lock, Eye } from "lucide-react";
import { cn } from "@/lib/utils";
import type { LspDocumentSymbol, LspSymbolKind } from "@/types/lsp";
import { getLspClient } from "@/lib/monaco-lsp";
import { generateStableUri } from "@/lib/monaco-lsp";

/**
 * Outline Panel Component
 *
 * Displays document symbols (functions, fields, accounts, events) in a tree view.
 * Updates in real-time as the active file changes.
 *
 * Features:
 * - Hierarchical symbol display (nested fields under accounts)
 * - Icon indicators for symbol kind (function, variable, type, etc.)
 * - Click to navigate to symbol location
 * - Auto-update when active file changes
 */
interface OutlinePanelProps {
  maxHeight?: string;
}

interface SymbolNode extends LspDocumentSymbol {
  isExpanded?: boolean;
  children?: SymbolNode[];
}

export default function OutlinePanel({ maxHeight = "calc(100vh - 400px)" }: OutlinePanelProps) {
  const { activeFile, files } = useIdeStore();
  const [symbols, setSymbols] = useState<SymbolNode[]>([]);
  const [expandedNodes, setExpandedNodes] = useState<Set<string>>(new Set());
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Load symbols when active file changes
  useEffect(() => {
    if (!activeFile) {
      setSymbols([]);
      return;
    }

    const loadSymbols = async () => {
      setIsLoading(true);
      setError(null);

      try {
        const lspClient = getLspClient();
        if (!lspClient) {
          setError("LSP not initialized");
          setSymbols([]);
          return;
        }

        const source = files[activeFile];
        if (!source) {
          setSymbols([]);
          return;
        }

        const uri = generateStableUri(activeFile);
        const symbolsJson = await lspClient.getDocumentSymbols(uri, source);

        if (symbolsJson) {
          const parsed = JSON.parse(symbolsJson) as SymbolNode[];
          // Initialize expansion state and add children
          const enhanced = enhanceSymbols(parsed, expandedNodes);
          setSymbols(enhanced);
        } else {
          setSymbols([]);
        }
      } catch (err) {
        console.error("[OutlinePanel] Error loading symbols:", err);
        setError(err instanceof Error ? err.message : "Failed to load symbols");
        setSymbols([]);
      } finally {
        setIsLoading(false);
      }
    };

    loadSymbols();
  }, [activeFile, files, expandedNodes]);

  const enhanceSymbols = (
    symbols: SymbolNode[],
    expanded: Set<string>
  ): SymbolNode[] => {
    return symbols.map((sym) => ({
      ...sym,
      isExpanded: expanded.has(getSymbolKey(sym)),
      children: sym.children ? enhanceSymbols(sym.children, expanded) : undefined,
    }));
  };

  const getSymbolKey = (symbol: LspDocumentSymbol): string => {
    return `${symbol.name}-${symbol.kind}-${symbol.range?.start?.line ?? 0}`;
  };

  const toggleExpanded = (symbol: SymbolNode) => {
    const key = getSymbolKey(symbol);
    const newExpanded = new Set(expandedNodes);

    if (newExpanded.has(key)) {
      newExpanded.delete(key);
    } else {
      newExpanded.add(key);
    }

    setExpandedNodes(newExpanded);
  };

  const navigateToSymbol = (symbol: SymbolNode) => {
    // This will integrate with editor navigation in Phase 2
    console.log(`[OutlinePanel] Navigate to symbol: ${symbol.name} at line ${symbol.range?.start?.line}`);
    // TODO: Implement actual editor navigation
  };

  const getSymbolIcon = (kind: LspSymbolKind | undefined) => {
    switch (kind) {
      case 6: // Function
        return <Function className="w-4 h-4" />;
      case 5: // Class
      case 23: // Interface
        return <Layers className="w-4 h-4" />;
      case 13: // Variable
      case 14: // Constant
        return <Circle className="w-4 h-4" />;
      case 4: // Enum
        return <Layers className="w-4 h-4" />;
      default:
        return <FileCode className="w-4 h-4" />;
    }
  };

  const renderSymbol = (symbol: SymbolNode, depth: number = 0): JSX.Element => {
    const key = getSymbolKey(symbol);
    const hasChildren = symbol.children && symbol.children.length > 0;

    return (
      <div key={key} className="select-none">
        <div
          className={cn(
            "flex items-center gap-2 px-3 py-1 hover:bg-accent cursor-pointer rounded text-sm",
            "transition-colors duration-150"
          )}
          style={{ marginLeft: `${depth * 12}px` }}
          onClick={() => {
            if (hasChildren) toggleExpanded(symbol);
            navigateToSymbol(symbol);
          }}
        >
          {hasChildren && (
            <button
              onClick={(e) => {
                e.stopPropagation();
                toggleExpanded(symbol);
              }}
              className="p-0 hover:bg-muted rounded"
            >
              {symbol.isExpanded ? (
                <ChevronDown className="w-4 h-4" />
              ) : (
                <ChevronRight className="w-4 h-4" />
              )}
            </button>
          )}
          {!hasChildren && <div className="w-4" />}

          <div className="flex items-center gap-2 flex-1 min-w-0">
            {getSymbolIcon(symbol.kind)}
            <span className="truncate text-foreground/80">{symbol.name}</span>
          </div>
        </div>

        {hasChildren && symbol.isExpanded && (
          <div>
            {symbol.children!.map((child) => renderSymbol(child, depth + 1))}
          </div>
        )}
      </div>
    );
  };

  return (
    <div className="flex flex-col gap-2 p-3 border-l border-border bg-background/50 h-full overflow-hidden">
      <div className="flex items-center justify-between flex-shrink-0">
        <h3 className="text-xs font-semibold text-foreground/70 uppercase tracking-wide">Outline</h3>
        {isLoading && <div className="text-xs text-muted-foreground">Loading...</div>}
      </div>

      <div
        className="flex-1 overflow-y-auto overflow-x-hidden space-y-1"
        style={{ maxHeight }}
      >
        {error && (
          <div className="text-xs text-destructive p-2 bg-destructive/10 rounded">
            {error}
          </div>
        )}

        {!error && symbols.length === 0 && !isLoading && (
          <p className="text-xs text-muted-foreground p-2">
            No symbols in this file
          </p>
        )}

        {symbols.map((symbol) => renderSymbol(symbol))}
      </div>
    </div>
  );
}
