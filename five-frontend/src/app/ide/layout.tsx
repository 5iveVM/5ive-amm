import { WalletProvider } from "@/components/providers/WalletProvider";

export default function IdeLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <WalletProvider>
      {children}
    </WalletProvider>
  );
}
