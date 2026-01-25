import dynamic from "next/dynamic";
import Hero from "@/components/landing/Hero";
import { ThemeToggle } from "@/components/ui/ThemeToggle";

// Rich Landing Page Sections (Dynamic Imports for performance)
const NapkinToMainnet = dynamic(() => import("@/components/landing/NapkinToMainnet"));

const SuperPowers = dynamic(() => import("@/components/landing/SuperPowers"));
const DeveloperExperience = dynamic(() => import("@/components/landing/DeveloperExperience"));

import Background from "@/components/layout/Background";

import Header from "@/components/layout/Header";

export default function LandingPage() {
    return (
        <div className="min-h-screen bg-transparent text-rose-pine-text font-sans selection:bg-rose-pine-love/30 flex flex-col relative overflow-x-hidden">
            {/* Full Page Grid */}
            <Background />

            {/* Command Capsule Header */}
            <Header />

            <main className="flex-1 relative z-10 w-full">
                <Hero />
                <NapkinToMainnet />

                <SuperPowers />
                <DeveloperExperience />
            </main>

            {/* Simple Footer */}
            {/* Simple Footer */}
            <footer className="py-8 border-t border-rose-pine-hl-low/20 text-center text-sm text-rose-pine-muted relative z-10">
                <p>© 2025 Five Org. All rights reserved.</p>
            </footer>
        </div>
    );
}
