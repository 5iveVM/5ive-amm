import { Connection, Keypair } from '@solana/web3.js';
import { readFileSync } from 'fs';
import { FiveSDK } from 'five-sdk';

const testDEXOnChain = async () => {
  try {
    console.log('=== Five DEX Protocol - On-Chain Execution Test ===\n');

    const rpcUrl = 'http://127.0.0.1:8900';
    const fiveVMProgramId = '9MHGM73eszNUtmJS6ypDCESguxWhCBnkUPpTMyLGqURH';
    const scriptAccount = 'Cdjp6JNT9U7wZZ928iRQ3TRHj58MytwTs8V54WDuM7Me';
    const vmStateAccount = 'FNRzA1gq3BYrkaxBxFRVJS26owGYSSfG3xjV45gimSUY';
    const keypairPath = '/Users/amberjackson/.config/solana/id.json';

    console.log('1. Configuration:');
    console.log(`   RPC URL: ${rpcUrl}`);
    console.log(`   FIVE VM Program: ${fiveVMProgramId}`);
    console.log(`   Script Account: ${scriptAccount}`);
    console.log(`   VM State Account: ${vmStateAccount}\n`);

    console.log('2. Setting up connection...');
    const connection = new Connection(rpcUrl);
    const slot = await connection.getSlot();
    console.log(`   ✓ Connected! Current slot: ${slot}\n`);

    console.log('3. Loading keypair...');
    const keypairData = JSON.parse(readFileSync(keypairPath, 'utf-8'));
    const keypair = Keypair.fromSecretKey(new Uint8Array(keypairData));
    console.log(`   ✓ Keypair loaded: ${keypair.publicKey.toString()}\n`);

    console.log('4. Using FiveSDK static method with correct program ID...');
    console.log(`   ✓ Will execute with FIVE VM Program: ${fiveVMProgramId}\n`);

    console.log('5. Executing test_simple() on-chain...');
    const result = await FiveSDK.executeOnSolana(
      scriptAccount,
      connection,
      keypair,
      0,
      [],
      [],
      {
        debug: true,
        fiveVMProgramId: fiveVMProgramId,
        network: 'localnet',
        computeUnitLimit: 1400000,
        maxRetries: 3,
        vmStateAccount: vmStateAccount
      }
    );

    console.log('\n6. Execution Result:');
    console.log(`   Success: ${result.success}`);
    console.log(`   Transaction ID: ${result.transactionId}`);
    if (result.error) {
      console.log(`   Error: ${result.error}`);
    }

    if (result.success) {
      console.log('\n✨ DEX protocol execution successful!');
      process.exit(0);
    } else {
      console.log('\n❌ Execution failed');
      process.exit(1);
    }

  } catch (error) {
    console.error('Error:', error.message);
    if (error.stack) console.error(error.stack);
    process.exit(1);
  }
};

testDEXOnChain();
