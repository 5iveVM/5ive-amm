import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  webpack: (config, { isServer }) => {
    if (!isServer) {
      config.resolve.fallback = {
        ...config.resolve.fallback,
        fs: false,
        path: false,
        os: false,
      };
    }
    config.experiments = {
      ...config.experiments,
      asyncWebAssembly: true,
    };

    // Suppress Webpack warnings about dynamic requires in WASM loaders and async WASM
    config.ignoreWarnings = [
      {
        module: /five-sdk/,
        message: /Critical dependency/,
      },
      {
        message: /The generated code contains 'async\/await'/,
      },
    ];

    return config;
  },
  output: 'export',
  images: {
    unoptimized: true,
  },
};

export default nextConfig;
