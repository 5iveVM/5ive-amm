"use client";

import { useMarketData } from "@/contexts/MarketDataContext";

/**
 * Legacy hook wrapper for MarketDataContext
 * Preserves API compatibility for existing components.
 */
export function useSolPrice() {
    return useMarketData();
}
