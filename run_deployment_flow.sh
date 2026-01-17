#!/bin/bash
set -e

# Function to kill background process on exit
cleanup() {
    if [ -n "$SURFPOOL_PID" ]; then
        echo "Stopping Surfpool (PID $SURFPOOL_PID)..."
        kill $SURFPOOL_PID || true
    fi
}
trap cleanup EXIT

# Kill any existing process on port 8899
echo "Checking for processes on port 8899..."
lsof -ti :8899 | xargs kill -9 || true
echo "Port 8899 cleared."

# Wait for surfpool binary (if build is still running)
echo "Waiting for surfpool binary to act as completed build artifact..."
while [ ! -f ./five-surfpool/surfpool ]; do
    echo "Waiting for build to finish... (checking ./five-surfpool/surfpool)"
    sleep 10
done

# Start Surfpool in background
echo "Starting Surfpool..."
FIVE_VALIDATOR=surfpool ./five-surfpool/surfpool start --no-tui -y > surfpool.log 2>&1 &
SURFPOOL_PID=$!
echo "Started Surfpool with PID $SURFPOOL_PID. Logs in surfpool.log"

# Wait for Surfpool to be ready
echo "Waiting for Surfpool to respond..."
max_retries=30
count=0
while ! curl -s http://127.0.0.1:8899/health > /dev/null; do
    sleep 1
    count=$((count+1))
    if [ $count -ge $max_retries ]; then
        echo "Surfpool failed to start in $max_retries seconds."
        cat surfpool.log
        exit 1
    fi
done
echo "Surfpool is ready!"

# Deploy and Init
echo "Running deploy-and-init.sh..."
./five-scripts/deploy-and-init.sh localnet ~/.config/solana/id.json prod > deploy.log 2>&1
cat deploy.log

# Extract Program ID and VM State PDA from summary (stripping colors)
PROGRAM_ID=$(grep "Program ID:" deploy.log | tail -n 1 | sed 's/\x1b\[[0-9;]*m//g' | awk '{print $3}' | tr -d '\r')
VM_STATE_PDA=$(grep "VM State PDA:" deploy.log | tail -n 1 | sed 's/\x1b\[[0-9;]*m//g' | awk '{print $4}' | tr -d '\r')

echo "Extracted Program ID: '$PROGRAM_ID'"
echo "Extracted VM State PDA: '$VM_STATE_PDA'"

if [ -z "$PROGRAM_ID" ] || [ -z "$VM_STATE_PDA" ]; then
    echo "Failed to extract deployment info"
    exit 1
fi


# Export variables for e2e-token-test.sh
export FIVE_PROGRAM_ID="$PROGRAM_ID"
export VM_STATE_PDA="$VM_STATE_PDA"

echo "Exported FIVE_PROGRAM_ID and VM_STATE_PDA."

# Run Token Test
echo "Running e2e-token-test.sh..."
cd five-templates/token
./e2e-token-test.sh --deploy
