"use client";

import React, { createContext, useContext, useState, useEffect } from "react";

interface MarketData {
    price: number | null;
    marketCap: number | null;
    loading: boolean;
}

const MarketDataContext = createContext<MarketData>({
    price: null,
    marketCap: null,
    loading: true,
});

export function MarketDataProvider({ children }: { children: React.ReactNode }) {
    const [data, setData] = useState<MarketData>({
        price: null,
        marketCap: null,
        loading: true,
    });

    useEffect(() => {
        const fetchData = async (retries = 2) => {
            for (let attempt = 0; attempt <= retries; attempt++) {
                try {
                    const response = await fetch(
                        'https://api.coingecko.com/api/v3/simple/price?ids=solana&vs_currencies=usd&include_market_cap=true'
                    );

                    if (response.ok) {
                        const json = await response.json();
                        if (json.solana?.usd) {
                            setData({
                                price: json.solana.usd,
                                marketCap: json.solana.usd_market_cap,
                                loading: false,
                            });
                            return;
                        }
                    }

                    // Non-OK response, retry after delay
                    if (attempt < retries) {
                        await new Promise(r => setTimeout(r, 2000 * (attempt + 1)));
                    }
                } catch (e) {
                    if (attempt < retries) {
                        await new Promise(r => setTimeout(r, 2000 * (attempt + 1)));
                    } else {
                        console.warn("Failed to fetch SOL price after retries", e);
                    }
                }
            }

            // All retries exhausted — keep loading state, no fake fallback
            setData(prev => ({ ...prev, loading: false }));
        };

        fetchData();

        // Refresh every 60s
        const interval = setInterval(() => fetchData(0), 60000);
        return () => clearInterval(interval);
    }, []);

    return (
        <MarketDataContext.Provider value={data}>
            {children}
        </MarketDataContext.Provider>
    );
}

export function useMarketData() {
    return useContext(MarketDataContext);
}
