import type { Metadata } from "next";
import { Inter, JetBrains_Mono, Noto_Sans_Myanmar } from "next/font/google";
import "./globals.css";

const inter = Inter({
  subsets: ["latin"],
  weight: ["300", "400", "500", "600", "700", "800", "900"],
  variable: "--font-inter",
  display: "swap",
});

const jetBrainsMono = JetBrains_Mono({
  subsets: ["latin"],
  weight: ["400", "500", "600"],
  variable: "--font-jetbrains-mono",
  display: "swap",
});

const notoSansMyanmar = Noto_Sans_Myanmar({
  subsets: ["myanmar"],
  weight: ["400", "600", "700"],
  variable: "--font-noto-sans-myanmar",
  display: "swap",
});

export const metadata: Metadata = {
  title: "M-Lang — Myanmar Language Compiler Presentation",
  description: "A Statically-Typed Compiler Using Romanized Burmese Keywords",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className={`${inter.variable} ${jetBrainsMono.variable} ${notoSansMyanmar.variable} font-sans antialiased`}>
        {children}
      </body>
    </html>
  );
}
