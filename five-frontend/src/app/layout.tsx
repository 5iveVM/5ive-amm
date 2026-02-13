import type { Metadata } from "next";
import { Onest, Geist_Mono } from "next/font/google";
import "./globals.css";
import { ThemeProvider } from "@/components/providers/ThemeProvider";
import { MarketDataProvider } from "@/contexts/MarketDataContext";

const onest = Onest({
  variable: "--font-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "5ive IDE",
  description: "Advanced IDE for 5ive Tech DSL",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body
        className={`${onest.variable} ${geistMono.variable} font-sans antialiased relative`}
        suppressHydrationWarning
      >
        <script
          dangerouslySetInnerHTML={{
            __html: `
              try {
                const storage = localStorage.getItem('five-theme-storage');
                if (storage) {
                  const parsed = JSON.parse(storage);
                  const theme = parsed.state?.theme;
                  if (theme) {
                    document.documentElement.classList.add(theme);
                    document.documentElement.style.colorScheme = theme;
                  }
                }
              } catch (e) {}
            `,
          }}
        />



        <ThemeProvider>
          <MarketDataProvider>
            {children}
          </MarketDataProvider>
        </ThemeProvider>
      </body>
    </html>
  );
}
