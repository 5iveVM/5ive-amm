#!/bin/bash
set -e

echo "Killing old validator..."
pkill -f solana-test-validator || true
sleep 2

echo "Starting Validator..."
solana-test-validator -r > validator.log 2>&1 &
echo "Validator PID: $!"
sleep 10

echo "Building Five Solana..."
cd ../../five-solana
cargo build-sbf

echo "Deploying Five Solana..."
solana program deploy target/deploy/five.so --program-id G7NFhT9ZBbrM1oqtNnWgd8mbB7A5FbbNt4XChvaPhA3A

echo "Running E2E Token Test..."
cd ../five-templates/token
./e2e-token-test.sh
