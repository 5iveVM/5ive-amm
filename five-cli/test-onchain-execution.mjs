#!/usr/bin/env node

/**
 * Test on-chain execution with varint encoding
 */

import { FiveSDK } from './dist/sdk/FiveSDK.js';
import { Connection, Keypair } from '@solana/web3.js';
import { readFile } from 'fs/promises';

async function testOnChainExecution() {
  console.log('🧪 Testing on-chain execution with varint encoding...');
  
  try {
    // Setup connection
    const connection = new Connection('http://localhost:8899', 'confirmed');
    console.log('✅ Connected to localnet');
    
    // Load keypair
    const keypairData = JSON.parse(await readFile('/Users/amberjackson/.config/solana/id.json', 'utf8'));
    const userKeypair = Keypair.fromSecretKey(new Uint8Array(keypairData));
    console.log(`✅ Loaded keypair: ${userKeypair.publicKey.toString()}`);
    
    // Execute the deployed script
    const scriptAccount = '7PDUTpsPt18y318JKSwhDv39wMkyAppZJjYhVEFSKiyj';
    const functionName = 0;  // Function index
    const parameters = [30, 40];  // Parameters: 30 + 40
    
    console.log(`🚀 Executing script account: ${scriptAccount}`);
    console.log(`📋 Function: ${functionName}, Parameters: [${parameters.join(', ')}]`);
    
    const executionResult = await FiveSDK.executeScriptAccount(
      scriptAccount,
      functionName,
      parameters,
      connection,
      userKeypair,
      {
        debug: true,
        network: 'local',
        computeBudget: 1400000
      }
    );
    
    console.log('\n🎯 EXECUTION RESULT:');
    console.log('Success:', executionResult.success);
    
    if (executionResult.success) {
      console.log('✅ Result:', executionResult.result);
      console.log('⚡ Compute Units Used:', executionResult.computeUnitsUsed);
      console.log('🆔 Transaction ID:', executionResult.transactionId);
      
      if (executionResult.logs) {
        console.log('\n📋 Execution Logs:');
        executionResult.logs.forEach(log => console.log(`  ${log}`));
      }
    } else {
      console.log('❌ Error:', executionResult.error);
      
      if (executionResult.logs) {
        console.log('\n📋 Error Logs:');
        executionResult.logs.forEach(log => console.log(`  ${log}`));
      }
    }
    
  } catch (error) {
    console.error('💥 Test failed:', error);
    process.exit(1);
  }
}

testOnChainExecution();