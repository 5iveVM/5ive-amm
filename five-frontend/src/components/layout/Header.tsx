"use client";

import { useState } from "react";
import Link from "next/link";
import { ThemeToggle } from "@/components/ui/ThemeToggle";
import { usePathname } from "next/navigation";
import { cn } from "@/lib/utils";
import { Menu, X } from "lucide-react";
import { AnimatePresence, motion } from "framer-motion";

export default function Header() {
    const pathname = usePathname();
    const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false);

    const navLinks = [
        { href: "/#features", label: "Features", color: "bg-rose-pine-rose" },
        { href: "/docs", label: "Docs", color: "bg-rose-pine-iris" },
        { href: "/ide", label: "IDE", color: "bg-rose-pine-gold" },
        { href: "/investor", label: "Investor", color: "bg-rose-pine-love" },
    ];

    return (
        <>
            <header className="fixed top-6 left-1/2 transform -translate-x-1/2 z-50 flex items-center justify-between px-6 py-3 rounded-full border border-rose-pine-hl-low/20 bg-rose-pine-surface/60 backdrop-blur-2xl shadow-[0_8px_32px_rgba(0,0,0,0.12)] w-[90%] max-w-5xl transition-all duration-500 hover:shadow-[0_8px_40px_rgba(0,0,0,0.2)] hover:border-rose-pine-hl-med/30">
                <div className="flex items-center gap-4">
                    <Link href="/" className="font-black text-xl tracking-tighter bg-gradient-to-b from-white via-[#c4a7e7] to-[#eb6f92] bg-clip-text text-transparent hover:opacity-80 transition-opacity" onClick={() => setIsMobileMenuOpen(false)}>
                        5IVE
                    </Link>
                </div>

                {/* Desktop Nav */}
                <nav className="hidden md:flex items-center gap-8 text-sm font-medium text-rose-pine-muted">
                    {navLinks.map((link) => (
                        <Link
                            key={link.href}
                            href={link.href}
                            className={cn(
                                "hover:text-rose-pine-text transition-colors relative group",
                                pathname === link.href && "text-rose-pine-text"
                            )}
                        >
                            {link.label}
                            <span className={cn(
                                "absolute -bottom-1 left-0 h-[1px] transition-all group-hover:w-full",
                                link.color,
                                pathname === link.href ? "w-full" : "w-0"
                            )} />
                        </Link>
                    ))}
                </nav>

                <div className="flex items-center gap-4">
                    <ThemeToggle />
                    {/* Github Link */}
                    <a href="https://github.com/five-org" target="_blank" rel="noopener noreferrer" className="text-rose-pine-muted hover:text-white transition-colors hidden md:block">
                        <span className="sr-only">GitHub</span>
                        <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                            <path fillRule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z" clipRule="evenodd" />
                        </svg>
                    </a>

                    {/* Mobile Menu Toggle */}
                    <button
                        onClick={() => setIsMobileMenuOpen(!isMobileMenuOpen)}
                        className="md:hidden p-2 text-rose-pine-muted hover:text-rose-pine-text transition-colors"
                    >
                        {isMobileMenuOpen ? <X className="w-6 h-6" /> : <Menu className="w-6 h-6" />}
                    </button>
                </div>
            </header>

            {/* Mobile Menu Overlay */}
            <AnimatePresence>
                {isMobileMenuOpen && (
                    <motion.div
                        initial={{ opacity: 0, y: -20, scale: 0.95 }}
                        animate={{ opacity: 1, y: 0, scale: 1 }}
                        exit={{ opacity: 0, y: -20, scale: 0.95 }}
                        transition={{ duration: 0.2 }}
                        className="fixed inset-x-4 top-24 z-40 p-6 rounded-3xl bg-[#191724]/95 backdrop-blur-3xl border border-rose-pine-hl-low/20 shadow-2xl md:hidden overflow-hidden"
                    >
                        <nav className="flex flex-col gap-4">
                            {navLinks.map((link, idx) => (
                                <motion.div
                                    key={link.href}
                                    initial={{ opacity: 0, x: -20 }}
                                    animate={{ opacity: 1, x: 0 }}
                                    transition={{ delay: idx * 0.05 }}
                                >
                                    <Link
                                        href={link.href}
                                        onClick={() => setIsMobileMenuOpen(false)}
                                        className={cn(
                                            "block p-4 rounded-xl text-lg font-bold transition-all border border-transparent",
                                            pathname === link.href
                                                ? "bg-white/5 text-rose-pine-text border-white/5"
                                                : "text-rose-pine-muted hover:bg-white/5 hover:text-rose-pine-text"
                                        )}
                                    >
                                        <div className="flex items-center justify-between">
                                            {link.label}
                                            {pathname === link.href && (
                                                <div className={`w-2 h-2 rounded-full ${link.color.replace('bg-', 'bg-')}`} />
                                            )}
                                        </div>
                                    </Link>
                                </motion.div>
                            ))}
                        </nav>

                        <div className="mt-8 pt-8 border-t border-white/5 flex justify-center">
                            <a href="https://github.com/five-org" target="_blank" rel="noopener noreferrer" className="flex items-center gap-2 text-rose-pine-muted hover:text-white transition-colors">
                                <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                                    <path fillRule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z" clipRule="evenodd" />
                                </svg>
                                <span className="text-sm">View Source on GitHub</span>
                            </a>
                        </div>
                    </motion.div>
                )}
            </AnimatePresence>
        </>
    );
}
