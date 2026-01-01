const { Connection, PublicKey } = require('@solana/web3.js');

const RPC_URL = "http://127.0.0.1:8899";
const PROGRAM_ID = "9MHGM73eszNUtmJS6ypDCESguxWhCBnkUPpTMyLGqURH";
const SCRIPT_ACCOUNT = "GvB7xAifdP5uBkSuDReuqQo3UoyMBPnNb45VD7CobrbZ";
const VM_STATE_PDA = "DRsZtpCF8Np1MsQixQPH4iQYTKhEkZMzNCTv15RCYys";

async function checkAccount(connection, label, pubkeyStr) {
    try {
        const pubkey = new PublicKey(pubkeyStr);
        const info = await connection.getAccountInfo(pubkey);
        console.log(`\n--- ${label} (${pubkeyStr}) ---`);
        if (!info) {
            console.log("Status: NOT FOUND");
            return;
        }
        console.log(`Owner: ${info.owner.toBase58()}`);
        console.log(`Expected Owner: ${PROGRAM_ID}`);
        console.log(`Data Length: ${info.data.length}`);

        if (info.owner.toBase58() !== PROGRAM_ID) {
            console.log("❌ INVALID OWNER");
        } else {
            console.log("✅ Owner Match");
        }

        // Peek at data start for magic bytes
        if (info.data.length >= 4) {
            const magic = info.data.slice(0, 4).toString();
            console.log(`Magic Bytes: ${info.data.subarray(0, 4)} ("${magic}")`);
        }
    } catch (e) {
        console.log(`Error checking ${label}:`, e.message);
    }
}

async function main() {
    console.log(`Connecting to ${RPC_URL}`);
    const connection = new Connection(RPC_URL, 'confirmed');

    await checkAccount(connection, "VM State PDA (Canonical)", VM_STATE_PDA);
    await checkAccount(connection, "Script Account (from logs)", SCRIPT_ACCOUNT);
}

main();
