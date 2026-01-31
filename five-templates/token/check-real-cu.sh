#!/bin/bash

RPC_URL="http://127.0.0.1:8899"

echo "═══════════════════════════════════════════════════════════════════"
echo "Checking REAL Transaction Data on Localnet"
echo "═══════════════════════════════════════════════════════════════════"
echo ""

# TX #1
echo "TX #1: Script Account Creation"
echo "Signature: gzQkRSXVYWaCK3AmYAgF3F8QvtVg5AdMirmS62bmrP7P62mjp8gKeBGHky9xeiiWPKSMpj5EkDiNraVHvj72EF7"
echo ""
curl -s -X POST $RPC_URL \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "getTransaction",
    "params": ["gzQkRSXVYWaCK3AmYAgF3F8QvtVg5AdMirmS62bmrP7P62mjp8gKeBGHky9xeiiWPKSMpj5EkDiNraVHvj72EF7", {"encoding": "json", "maxSupportedTransactionVersion": 0}]
  }' | jq '.result | if .meta then {status: .meta.err, cu: .meta.computeUnitsConsumed, logs: .meta.logMessages[0:5]} else "Transaction not found" end'

echo ""
echo "─────────────────────────────────────────────────────────────────"
echo ""

# TX #2
echo "TX #2: Mint Account Creation"
echo "Signature: 2HTvikVbWM94PLqrJcmwsGg61nFwYFL7KmGk1q4HDMeKBEanjNBB5NbJU1qLZEd5HTJ4eNRj1Mrs8Pz7aCt1MrJE"
echo ""
curl -s -X POST $RPC_URL \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "getTransaction",
    "params": ["2HTvikVbWM94PLqrJcmwsGg61nFwYFL7KmGk1q4HDMeKBEanjNBB5NbJU1qLZEd5HTJ4eNRj1Mrs8Pz7aCt1MrJE", {"encoding": "json", "maxSupportedTransactionVersion": 0}]
  }' | jq '.result | if .meta then {status: .meta.err, cu: .meta.computeUnitsConsumed, logs: .meta.logMessages[0:5]} else "Transaction not found" end'

echo ""
echo "─────────────────────────────────────────────────────────────────"
echo ""

# TX #3
echo "TX #3: Token Account Creation"
echo "Signature: 2Tj3ggGFPbXxL71njxCboEKLuRf8D4eTP97KkXCTzE7LMLtsU4dS3U2mYnu8hbLSs8uNcXN9NuWRj2FmHT2H6cF1"
echo ""
curl -s -X POST $RPC_URL \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "getTransaction",
    "params": ["2Tj3ggGFPbXxL71njxCboEKLuRf8D4eTP97KkXCTzE7LMLtsU4dS3U2mYnu8hbLSs8uNcXN9NuWRj2FmHT2H6cF1", {"encoding": "json", "maxSupportedTransactionVersion": 0}]
  }' | jq '.result | if .meta then {status: .meta.err, cu: .meta.computeUnitsConsumed, logs: .meta.logMessages[0:5]} else "Transaction not found" end'

echo ""
echo "═══════════════════════════════════════════════════════════════════"
