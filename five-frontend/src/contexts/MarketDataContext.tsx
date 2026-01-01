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
        const fetchData = async () => {
            try {
                // Fetch SOL Price + Market Cap from CoinGecko
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

                // Fallback on error/empty response
                console.warn("Using fallback market data.");
                setData({
                    price: 200,
                    marketCap: 120000000000, // $120B Fallback
                    loading: false,
                });

            } catch (e) {
                // Slient fail, use fallback
                setData({
                    price: 200,
                    marketCap: 120000000000,
                    loading: false,
                });
            }
        };

        fetchData();

        // Refresh every 60s
        const interval = setInterval(fetchData, 60000);
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
