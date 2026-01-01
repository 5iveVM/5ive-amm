"use client";

import { useState } from "react";
import { Folder, Play, Rocket, Settings, ChevronLeft, ChevronRight, PanelLeftClose, PanelLeftOpen, Book } from "lucide-react";
import ProjectExplorer from "@/components/editor/ProjectExplorer";
import ExecutionControls from "@/components/ide/ExecutionControls";
import VMVisualizer from "@/components/vm/VMVisualizer";
import DeployManager from "@/components/deploy/DeployManager";
import ScriptBrowser from "@/components/editor/ScriptBrowser";
import { cn } from "@/lib/utils";

interface AppSidebarProps {
    isOpen: boolean;
    onToggle: () => void;
    onCompile: (path?: string) => void;
    activeTab: Tab;
    onTabChange: (tab: Tab) => void;

    // Execution Props
    onRun: () => void;
    isExecuting: boolean;
    isOnChain: boolean;
    onToggleMode: (isOnChain: boolean) => void;
    estimatedCost: number | null;
    solPrice: number;
}

type Tab = 'files' | 'run' | 'deploy' | 'examples';

export default function AppSidebar({
    isOpen,
    onToggle,
    onCompile,
    activeTab,
    onTabChange,
    onRun,
    isExecuting,
    isOnChain,
    onToggleMode,
    estimatedCost,
    solPrice
}: AppSidebarProps) {

    return (
        <div
            className={cn(
                // Base: Fixed position
                "fixed transition-all duration-500 ease-out shadow-2xl flex flex-col overflow-hidden bg-rose-pine-surface/95 backdrop-blur-2xl border-r border-white/10 z-40",
                // Mobile Styles: Full height drawer, slide from left. h-[100dvh] for mobile browsers.
                "h-[100dvh] top-0 left-0 w-[85vw] max-w-[320px] rounded-r-2xl border-y-0 pb-safe",
                // Desktop Styles: Floating capsule
                "md:h-auto md:top-24 md:bottom-6 md:left-4 md:w-80 md:rounded-3xl md:border md:bg-rose-pine-surface/60 md:pb-0",
                // State Toggle
                isOpen ? "translate-x-0 opacity-100 scale-100" : "-translate-x-full opacity-0 md:-translate-x-[120%] md:scale-95"
            )}
        >
            {/* Header with Title and Close Button */}
            <div className={cn(
                "flex items-center justify-between px-5 py-4 border-b border-white/5 shrink-0 bg-white/5 mt-14 md:mt-0",
                "md:rounded-t-3xl" // Match parent rounded corners on desktop
            )}>
                <span className="text-sm font-bold text-rose-pine-text capitalize flex items-center gap-2">
                    {activeTab === 'files' && <><Folder size={16} className="text-rose-pine-iris" /> Project Files</>}
                    {activeTab === 'run' && <><Play size={16} className="text-rose-pine-iris" /> Run & Debug</>}
                    {activeTab === 'deploy' && <><Rocket size={16} className="text-rose-pine-iris" /> Deployment</>}
                    {activeTab === 'examples' && <><Book size={16} className="text-rose-pine-iris" /> Examples</>}
                </span>

                <button
                    onClick={onToggle}
                    className="p-1.5 rounded-md text-rose-pine-muted hover:text-rose-pine-text hover:bg-white/5 transition-colors md:flex hidden"
                >
                    <PanelLeftClose size={16} />
                </button>
            </div>

            {/* Mobile Tab Navigation (Visible only on mobile inside the drawer) */}
            <div className="flex md:hidden items-center p-2 gap-1 border-b border-white/5 overflow-x-auto shrink-0">
                {(['files', 'run', 'deploy', 'examples'] as Tab[]).map((tab) => (
                    <button
                        key={tab}
                        onClick={() => onTabChange(tab)}
                        className={cn(
                            "flex-1 flex flex-col items-center justify-center py-2 px-1 rounded-lg text-[10px] font-medium transition-all gap-1",
                            activeTab === tab
                                ? "bg-rose-pine-iris/20 text-rose-pine-iris"
                                : "text-rose-pine-subtle hover:bg-white/5"
                        )}
                    >
                        {tab === 'files' && <Folder size={16} />}
                        {tab === 'run' && <Play size={16} />}
                        {tab === 'deploy' && <Rocket size={16} />}
                        {tab === 'examples' && <Book size={16} />}
                        <span className="capitalize">{tab}</span>
                    </button>
                ))}
            </div>


            {/* Content Area */}
            <div className="flex-1 overflow-hidden relative pb-8 md:pb-0">
                {/* Files Tab */}
                {activeTab === 'files' && (
                    <div className="h-full flex flex-col animate-in fade-in slide-in-from-left-4 duration-200">
                        <ProjectExplorer onCompile={onCompile} />
                    </div>
                )}

                {/* Run Tab */}
                {activeTab === 'run' && (
                    <div className="h-full flex flex-col animate-in fade-in slide-in-from-left-4 duration-200 overflow-y-auto custom-scrollbar">
                        <ExecutionControls
                            onRun={onRun}
                            isExecuting={isExecuting}
                            isOnChain={isOnChain}
                            onToggleMode={onToggleMode}
                            estimatedCost={estimatedCost}
                            solPrice={solPrice}
                        />
                        <div className="flex-1 min-h-[300px]">
                            <VMVisualizer />
                        </div>
                    </div>
                )}

                {/* Deploy Tab */}
                {activeTab === 'deploy' && (
                    <div className="h-full flex flex-col animate-in fade-in slide-in-from-left-4 duration-200 p-3 pt-0">
                        <DeployManager />
                    </div>
                )}

                {/* Examples Tab */}
                {activeTab === 'examples' && (
                    <div className="h-full flex flex-col animate-in fade-in slide-in-from-left-4 duration-200">
                        <ScriptBrowser onLoad={() => {
                            // Optional: Switch to Files tab after loading? 
                            // For now let's keep it here so they can explore more.
                            onTabChange('files');
                        }} />
                    </div>
                )}
            </div>


        </div>
    );
}
