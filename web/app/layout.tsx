import type { Metadata } from "next";
import { Barlow } from "next/font/google";

import "./globals.css";

const barlow = Barlow({
  variable: "--font-barlow",
  subsets: ["latin"],
  weight: ["400", "600", "700"],
});

export const metadata: Metadata = {
  title: "Lending simulation",
  description: "Watch real market prices move a simulated loan toward liquidation.",
};

export default function RootLayout({ children }: LayoutProps<"/">) {
  return (
    <html lang="en" className={`${barlow.variable} antialiased`}>
      <body>{children}</body>
    </html>
  );
}
