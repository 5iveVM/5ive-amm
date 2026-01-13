import { Connection, PublicKey } from '@solana/web3.js';

const RPC_URL = 'http://127.0.0.1:8899';
const connection = new Connection(RPC_URL, 'confirmed');

const accounts = {
  counter1: 'EuGDHEw4jk6DzTvh46URxWy1tJHEWahtCfEV17uR1AwM',
  counter2: 'BS6DzrYzXaFQS9a1mKjHvgBzhxatTYqpyWpmgmcwq77C'
};

async function checkCounter(pubkey, label) {
    const account = await connection.getAccountInfo(new PublicKey(pubkey));
    if (!account) {
        console.log(`${label}: NOT FOUND`);
        return;
    }
    
    // Counter struct:
    // owner: pubkey (32)
    // count: u64 (8)
    const count = Number(account.data.readBigUInt64LE(32));
    const owner = new PublicKey(account.data.subarray(0, 32)).toBase58();
    
    console.log(`${label}:`);
    console.log(`  Address: ${pubkey}`);
    console.log(`  Owner:   ${owner}`);
    console.log(`  Count:   ${count}`);
}

async function main() {
    console.log('=== Counter State Verification ===');
    await checkCounter(accounts.counter1, 'Counter 1 (User 1)');
    await checkCounter(accounts.counter2, 'Counter 2 (User 2)');
}

main().catch(console.error);
