import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  // The browser only talks to Next, which relays to the Rust API: no CORS needed.
  async rewrites() {
    return [
      { source: "/api/:path*", destination: "http://127.0.0.1:3000/:path*" },
    ];
  },
  cacheComponents: true,
  partialPrefetching: true,
  turbopack: {
    rules: {
      "*.css": {
        loaders: ["@tailwindcss/turbopack"],
        as: "*.css",
      },
    },
  },
};

export default nextConfig;
