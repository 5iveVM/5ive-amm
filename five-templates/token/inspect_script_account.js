
const { Connection, PublicKey } = require('@solana/web3.js');
const fs = require('fs');

async function main() {
    const rpcUrl = 'http://127.0.0.1:8899';
    const connection = new Connection(rpcUrl, 'confirmed');
    const scriptPubkey = new PublicKey(process.argv[2] || '5opYNqa5wxrC7SpdqsKHVcNqtAx1tDSmwJeyAdwFZ2qE');

    console.log(`Inspecting account: ${scriptPubkey.toBase58()}`);

    const info = await connection.getAccountInfo(scriptPubkey);
    if (!info) {
        console.log('Account not found');
        return;
    }

    const rent = await connection.getMinimumBalanceForRentExemption(info.data.length);
    console.log(`Lamports: ${info.lamports}`);
    console.log(`Rent Exempt Min (for ${info.data.length}): ${rent}`);
    console.log(`Rent Status: ${info.lamports >= rent ? 'OK' : 'UNDERFUNDED'}`);

    console.log(`Data length: ${info.data.length}`);
    const header = info.data.slice(0, 64);

    // Dump header bytes hex
    console.log('Header (hex):', header.toString('hex'));

    // Parse specific fields
    const magic = header.slice(0, 4).toString();
    console.log(`Magic: ${magic}`);

    const version = header[4];
    console.log(`Version: ${version}`);

    const permissions = header[5];
    console.log(`Permissions: ${permissions}`);

    // Reserved1 is at offset 58 (64 - 6)
    // Actually struct layout:
    // magic: 4
    // version: 1
    // permissions: 1
    // reserved0: 2
    // owner: 32
    // script_id: 8
    // bytecode_len: 4
    // metadata_len: 4
    // func_count: 2
    // reserved1: 6  <-- here. Offset: 4+1+1+2+32+8+4+4+2 = 58.

    const reserved1 = header.slice(58, 64);
    console.log('Reserved1:', reserved1.toString('hex'));

    const upload_len = reserved1.readUInt32LE(0);
    const upload_complete = reserved1[4];
    const upload_mode = reserved1[5];

    console.log(`Upload Len: ${upload_len}`);
    console.log(`Upload Complete: ${upload_complete}`);
    console.log(`Upload Mode: ${upload_mode}`);
}

main().catch(console.error);
