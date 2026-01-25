"use client";

function Background({ children }: { children?: React.ReactNode }) {
    return (
        <div className="fixed inset-0 w-full h-full overflow-hidden bg-rose-pine-base z-0 pointer-events-none transition-colors duration-500">
            {/* Luminous Fluidity Background */}
            <div className="absolute inset-0 z-0">
                <div className="absolute inset-0 bg-rose-pine-base transition-colors duration-500" />
                <div className="absolute inset-0 overflow-hidden">
                    {/* Orb 1: Rose (Love) - Top Left */}
                    <div
                        className="absolute top-[-10%] left-[-10%] w-[800px] h-[800px] bg-rose-pine-love rounded-full mix-blend-multiply dark:mix-blend-screen blur-[120px] opacity-40 dark:opacity-20 animate-orb-1"
                    />

                    {/* Orb 2: Iris (Gold) - Top Right */}
                    <div
                        className="absolute top-[-20%] right-[-10%] w-[900px] h-[900px] bg-rose-pine-iris rounded-full mix-blend-multiply dark:mix-blend-screen blur-[120px] opacity-40 dark:opacity-20 animate-orb-2"
                    />

                    {/* Orb 3: Pine (Blue) - Bottom Left */}
                    <div
                        className="absolute bottom-[-20%] left-[-10%] w-[1000px] h-[1000px] bg-rose-pine-pine rounded-full mix-blend-multiply dark:mix-blend-screen blur-[130px] opacity-40 dark:opacity-20 animate-orb-3"
                    />

                    {/* Orb 4: Foam (Teal) - Bottom Right */}
                    <div
                        className="absolute bottom-[-10%] right-[-20%] w-[800px] h-[800px] bg-rose-pine-foam rounded-full mix-blend-multiply dark:mix-blend-screen blur-[120px] opacity-40 dark:opacity-20 animate-orb-4"
                    />

                    {/* Orb 5: Center Interaction Orb (Gold) */}
                    <div
                        className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[600px] bg-rose-pine-gold rounded-full mix-blend-normal dark:mix-blend-screen blur-[100px] opacity-30 dark:opacity-10 animate-orb-center"
                    />

                </div>
            </div>

            {/* Content Layer */}
            <div className="relative z-10 w-full h-full">
                {children}
            </div>
        </div>
    );
}

export default Background;
