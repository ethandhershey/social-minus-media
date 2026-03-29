import { Geist, Geist_Mono } from "next/font/google";
import "./globals.css";
// import { absoluteUrl } or use env
// metadataBase: new URL("https://your-domain.com"),

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata = {
  title: "GatheRound",
  description: "Post activities and meet people in real life.",
  applicationName: "GatheRound",
  appleWebApp: {
    capable: true,
    title: "GatheRound",
    statusBarStyle: "default",
  },
  formatDetection: {
    telephone: false,
  },
  icons: {
    apple: "/gatheroundicon2.svg",
  },
};

export const viewport = {
  width: "device-width",
  initialScale: 1,
  maximumScale: 5,
  themeColor: "#7dae8a",
};

export default function RootLayout({ children }) {
  return (
    <html lang="en" className={`${geistSans.variable} ${geistMono.variable}`}>
      <body>{children}</body>
    </html>
  );
}
