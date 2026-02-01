#!/bin/bash
set -e

echo "🚀 Deploying Register-Optimized Token Bytecode"
echo "=================================================="

# Backup original artifact if it exists
if [ -f build/five-token-template.five ]; then
    echo "Backing up original artifact..."
    cp build/five-token-template.five build/five-token-template-baseline.five
fi

# Copy register artifact to the expected location
echo "Using register-optimized bytecode..."
cp build/five-token-registers.five build/five-token-template.five

# Run deployment
echo "Running deployment..."
npm run deploy

# Restore original artifact
if [ -f build/five-token-template-baseline.five ]; then
    echo "Restoring original artifact..."
    cp build/five-token-template-baseline.five build/five-token-template.five
fi

echo "✅ Deployment complete!"
