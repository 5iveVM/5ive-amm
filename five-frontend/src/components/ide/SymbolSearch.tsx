"use client";

import { useEffect, useState, useRef } from "react";
import { Search, Loader2, Zap, FileCode } from "lucide-react";
import { cn } from "@/lib/utils";
import type { LspSymbolInformation, LspSymbolKind } from "@/types/lsp";
import { getLspClient } from "@/lib/monaco-lsp";
import { generateStableUri } from "@/lib/monaco-lsp";

/**
 * Symbol Search Component (Cmd+T / Ctrl+T)
 *
 * Provides fuzzy search across all workspace symbols.
 *
 * Features:
 * - Real-time search as user types
 * - Shows file path + symbol kind
 * - Navigate to symbol on selection
 * - Keyboard navigation (arrow keys, Enter, Esc)
 * - Cmd+T / Ctrl+T keybinding
 */
interface SymbolSearchProps {
  onNavigate?: (symbol: LspSymbolInformation) => void;
  isOpen?: boolean;
  onClose?: () => void;
}

export default function SymbolSearch({
  onNavigate,
  isOpen = false,
  onClose,
}: SymbolSearchProps) {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<LspSymbolInformation[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [isSearching, setIsSearching] = useState(false);
  const [isOpen_, setIsOpen_] = useState(isOpen);
  const searchInputRef = useRef<HTMLInputElement>(null);

  // Handle Cmd+T / Ctrl+T keybinding
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "t") {
        e.preventDefault();
        setIsOpen_(!isOpen_);
        if (!isOpen_) {
          setTimeout(() => searchInputRef.current?.focus(), 0);
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isOpen_]);

  // Focus input when opened
  useEffect(() => {
    if (isOpen_) {
      setTimeout(() => searchInputRef.current?.focus(), 0);
    }
  }, [isOpen_]);

  // Search workspace symbols
  useEffect(() => {
    if (!query) {
      setResults([]);
      setSelectedIndex(0);
      return;
    }

    const performSearch = async () => {
      setIsSearching(true);
      try {
        const lspClient = getLspClient();
        if (!lspClient) {
          setResults([]);
          return;
        }

        const symbols = await lspClient.getWorkspaceSymbols(query);
        setResults(symbols);
        setSelectedIndex(0);
      } catch (err) {
        console.error("[SymbolSearch] Error searching symbols:", err);
        setResults([]);
      } finally {
        setIsSearching(false);
      }
    };

    const debounceTimer = setTimeout(performSearch, 200);
    return () => clearTimeout(debounceTimer);
  }, [query]);

  // Handle keyboard navigation
  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    switch (e.key) {
      case "ArrowDown":
        e.preventDefault();
        setSelectedIndex((prev) => Math.min(prev + 1, results.length - 1));
        break;
      case "ArrowUp":
        e.preventDefault();
        setSelectedIndex((prev) => Math.max(prev - 1, 0));
        break;
      case "Enter":
        e.preventDefault();
        if (results[selectedIndex]) {
          handleSelectSymbol(results[selectedIndex]);
        }
        break;
      case "Escape":
        e.preventDefault();
        setIsOpen_(false);
        onClose?.();
        break;
    }
  };

  const handleSelectSymbol = (symbol: LspSymbolInformation) => {
    onNavigate?.(symbol);
    setIsOpen_(false);
    onClose?.();
    setQuery("");
  };

  const getSymbolIcon = (kind: LspSymbolKind | undefined) => {
    const iconProps = "w-4 h-4 text-muted-foreground";
    switch (kind) {
      case 6: // Function
        return <Zap className={iconProps} />;
      case 5: // Class
      case 23: // Interface
        return <FileCode className={iconProps} />;
      default:
        return <FileCode className={iconProps} />;
    }
  };

  const formatPath = (uri: string): string => {
    // Extract relative path from file:///workspace/...
    const match = uri.match(/file:\/\/\/workspace\/(.*)/);
    return match ? match[1] : uri;
  };

  if (!isOpen_) return null;

  return (
    <>
      {/* Overlay */}
      <div
        className="fixed inset-0 bg-black/50 z-40"
        onClick={() => {
          setIsOpen_(false);
          onClose?.();
        }}
      />

      {/* Search Dialog */}
      <div className="fixed top-[20%] left-1/2 -translate-x-1/2 w-full max-w-2xl z-50">
        <div className="bg-popover border border-border rounded-lg shadow-lg overflow-hidden">
          {/* Search Input */}
          <div className="flex items-center gap-2 px-4 py-3 border-b border-border">
            <Search className="w-5 h-5 text-muted-foreground flex-shrink-0" />
            <input
              ref={searchInputRef}
              type="text"
              placeholder="Search symbols (Cmd+T)..."
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onKeyDown={handleKeyDown}
              className={cn(
                "flex-1 bg-transparent outline-none text-foreground",
                "placeholder:text-muted-foreground text-sm"
              )}
            />
            {isSearching && <Loader2 className="w-4 h-4 animate-spin text-muted-foreground" />}
          </div>

          {/* Results */}
          <div className="max-h-96 overflow-y-auto">
            {results.length === 0 && query && !isSearching && (
              <div className="px-4 py-8 text-center text-sm text-muted-foreground">
                No symbols found for "{query}"
              </div>
            )}

            {results.length === 0 && !query && !isSearching && (
              <div className="px-4 py-8 text-center text-sm text-muted-foreground">
                Start typing to search...
              </div>
            )}

            {results.map((symbol, index) => (
              <div
                key={`${symbol.name}-${symbol.location.uri}-${index}`}
                onClick={() => handleSelectSymbol(symbol)}
                className={cn(
                  "flex items-center gap-3 px-4 py-3 cursor-pointer transition-colors",
                  "border-b border-border last:border-b-0",
                  index === selectedIndex
                    ? "bg-accent text-accent-foreground"
                    : "hover:bg-muted"
                )}
              >
                <div className="flex items-center gap-2 flex-shrink-0">
                  {getSymbolIcon(symbol.kind)}
                </div>

                <div className="flex-1 min-w-0">
                  <div className="font-medium text-sm truncate">
                    {symbol.name}
                  </div>
                  <div className="text-xs text-muted-foreground truncate">
                    {formatPath(symbol.location.uri)}
                  </div>
                </div>

                <div className="flex-shrink-0 text-xs text-muted-foreground">
                  {symbol.location.range?.start?.line
                    ? `${symbol.location.range.start.line + 1}:${symbol.location.range.start.character}`
                    : ""}
                </div>
              </div>
            ))}
          </div>

          {/* Footer */}
          {results.length > 0 && (
            <div className="px-4 py-2 border-t border-border bg-muted/30 text-xs text-muted-foreground">
              Press Enter to go, ↑↓ to navigate, Esc to close
            </div>
          )}
        </div>
      </div>
    </>
  );
}
