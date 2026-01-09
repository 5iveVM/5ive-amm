
const { PublicKey } = require('@solana/web3.js');

const PROGRAM_ID = new PublicKey('FJERHDufQjbHvXjYuoprJQhKY4413cTPKhbbNSSw4tBg');
const SEED = Buffer.from("five_vm_state");

async function main() {
    const [pda] = await PublicKey.findProgramAddress([SEED], PROGRAM_ID);
    console.log("VM State PDA:", pda.toBase58());
}

main();
