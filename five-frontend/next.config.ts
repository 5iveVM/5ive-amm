import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  transpilePackages: ["five-sdk"],
  experimental: {
    esmExternals: 'loose',
  },
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
      layers: true,
    };

    // Fix for "Cannot use 'import.meta' outside a module"
    // and ensuring we can handle the CommonJS WASM glue in an ESM world
    config.module.rules.push({
      test: /five_vm_wasm\.js$/,
      type: 'javascript/auto',
    });

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
