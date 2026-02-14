import { Connection } from '@solana/web3.js';

const localnetConnection = new Connection('http://127.0.0.1:8899', 'confirmed');
const devnetConnection = new Connection('https://api.devnet.solana.com', 'confirmed');

const localnetSigs = [
  '3oRkCifaLvfC7NiW2RurrETAYKVDN4pL1Fm7W4AQAkrrHH4puRzifaJTM1FLk6JPvXDKxQBZm2SyeTiZ9EPfr4W2',
  '25sST87eCqChE8GDhxKAjTtRFi3wLhgYnMc1mnD4ALhh8MJ1bVpdxTrMkFcmcQ18zzCMy7c76tiVwXTPVf7QxGS2',
  '3rL8bJhn6PHmFpkCgh8aoXhnXFcjhfs7UVMGfdegC12Az9B8N4iGHzZfmxPo8vNuJRMQGsDHgNo9Fih2Y9TtRZyb'
];

const devnetSigs = [
  '3UPVCYtcWW4bw4pZCPVxw7zecXhcA3yDzisFrjzusNF2EyuNsp7usskXV5zUo6w6QTfPmRrnEaxxKLZPMrkUH9WA',
  '4yUbGNbUDq2jXQ2q7RkQC2G5BvL6pAWQTEVzon5rMD6CsLMwNAF8AgvFMKo2ytx6MRJrBv3Dfsw8o9xcQ5cZAT8A',
  '2JH4P7Vgg6LmjtRoTtavzamku2muG678kEnkkMHgE7uCPtNV7tkuGTuNYDsgCNrdhH5X2aFmEAtpueNMY2eH49oj'
];

const labels = ['Initialize Mint', 'Initialize Token Account', 'Mint To'];

async function analyzeTx(sig, label, network) {
  const connection = network === 'localnet' ? localnetConnection : devnetConnection;
  const tx = await connection.getTransaction(sig, { commitment: 'confirmed', maxSupportedTransactionVersion: 0 });
  
  if (!tx) {
    console.log(`${label} (${network}): Transaction not found`);
    return;
  }

  const meta = tx.meta;
  const message = tx.transaction.message;
  
  console.log(`\n${'='.repeat(80)}`);
  console.log(`${label} - ${network.toUpperCase()}`);
  console.log(`${'='.repeat(80)}`);
  console.log(`Signature: ${sig}`);
  console.log(`Status: ${meta.err ? 'FAILED' : 'SUCCESS'}`);
  console.log(`CU Consumed: ${meta.computeUnitsConsumed}`);
  console.log(`CU Budget: ${meta.computeUnitsConsumed} (from logs)`);
  console.log(`Accounts: ${message.accountKeys.length}`);
  console.log(`Instructions: ${message.compiledInstructions.length}`);
  console.log(`Fee: ${meta.fee} lamports`);
  console.log(`Log Messages:`);
  if (meta.logMessages) {
    meta.logMessages.forEach(log => {
      if (log.includes('invoke') || log.includes('success') || log.includes('failed') || log.includes('consumed')) {
        console.log(`  ${log}`);
      }
    });
  }
}

// Analyze all 6 transactions
console.log('\n\n🔍 DETAILED TRANSACTION ANALYSIS\n');

for (let i = 0; i < labels.length; i++) {
  await analyzeTx(localnetSigs[i], labels[i], 'localnet');
  await analyzeTx(devnetSigs[i], labels[i], 'devnet');
}
