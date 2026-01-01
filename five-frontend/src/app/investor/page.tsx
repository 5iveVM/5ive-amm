"use client";

import Header from "@/components/layout/Header";
import Background from "@/components/layout/Background";
import HeroSection from "@/components/investor/HeroSection";
import AIAdvantageSection from "@/components/investor/AIAdvantageSection";
import EconomicDeepDive from "@/components/investor/EconomicDeepDive";
import ReplacementEvent from "@/components/investor/ReplacementEvent";
import RuntimeDeepDive from "@/components/investor/RuntimeDeepDive";
import TokenSection from "@/components/investor/TokenSection";
import AgentEconomy from "@/components/investor/AgentEconomy";
import CTASection from "@/components/investor/CTASection";
import { MarketDataProvider } from "@/contexts/MarketDataContext";

export default function InvestorPage() {
  return (
    <div className="min-h-screen bg-transparent text-rose-pine-text font-sans selection:bg-rose-pine-love/30 flex flex-col relative overflow-x-hidden">
      <MarketDataProvider>
        <Background />
        <Header />

        <main className="flex-1 relative z-10 w-full flex flex-col items-center">
          <HeroSection />

          <div id="ai" className="w-full">
            <AIAdvantageSection />
          </div>

          <div id="economics" className="w-full">
            <EconomicDeepDive />
          </div>

          <div id="adapt" className="w-full">
            <ReplacementEvent />
          </div>

          <div id="engine" className="w-full">
            <RuntimeDeepDive />
          </div>

          <div id="agents" className="w-full">
            <AgentEconomy />
          </div>

          <div id="tokenomics" className="w-full">
            <TokenSection />
          </div>

          <CTASection />
        </main>

        <footer className="py-8 border-t border-rose-pine-hl-low/20 text-center text-sm text-rose-pine-muted relative z-10 w-full">
          <p>© 2025 Five Org. All rights reserved.</p>
        </footer>
      </MarketDataProvider>
    </div>
  );
}
