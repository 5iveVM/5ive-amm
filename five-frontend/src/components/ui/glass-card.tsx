import { cn } from "@/lib/utils";
import { ReactNode } from "react";

interface GlassCardProps extends React.HTMLAttributes<HTMLDivElement> {
    children: ReactNode;
    variant?: "default" | "heavy" | "input";
    hoverEffect?: boolean;
}

export function GlassCard({ children, className, variant = "default", hoverEffect = false, ...props }: GlassCardProps) {
    return (
        <div
            className={cn(
                "transition-all duration-300",
                variant === "default" && "glass-panel rounded-xl",
                variant === "heavy" && "glass-panel-heavy rounded-2xl",
                variant === "input" && "glass-input",
                hoverEffect && "hover:shadow-[0_0_20px_rgba(234,154,151,0.1)] hover:border-white/10",
                className
            )}
            {...props}
        >
            {children}
        </div>
    );
}

export function GlassHeader({ title, children, className }: { title?: string; children?: ReactNode; className?: string }) {
    return (
        <div className={cn("flex items-center justify-between p-4 border-b border-white/5", className)}>
            {title && <h3 className="font-medium text-rose-pine-text/90 tracking-wide">{title}</h3>}
            {children}
        </div>
    );
}
