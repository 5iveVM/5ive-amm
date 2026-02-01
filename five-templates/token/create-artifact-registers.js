const fs = require('fs');
const path = require('path');

const binPath = 'build/token-registers.fbin';
const abiPath = 'src/token.abi.json';
const outPath = 'build/five-token-registers.five';

if (!fs.existsSync(binPath)) {
    console.error(`Missing bytecode file: ${binPath}`);
    console.error("Please run: cargo run --bin five -- compile ../five-templates/token/src/token.v --enable-registers --use-linear-scan --output ../five-templates/token/build/token-registers.fbin");
    process.exit(1);
}

if (!fs.existsSync(abiPath)) {
    console.error(`Missing ABI file: ${abiPath}`);
    process.exit(1);
}

const bytecode = fs.readFileSync(binPath);
const abi = JSON.parse(fs.readFileSync(abiPath, 'utf8'));

const artifact = {
    name: "five-token-registers",
    bytecode: bytecode.toString('base64'),
    abi: abi
};

if (!fs.existsSync('build')) {
    fs.mkdirSync('build');
}

fs.writeFileSync(outPath, JSON.stringify(artifact, null, 2));
console.log(`Register-optimized artifact created at ${outPath}`);
console.log(`Bytecode size: ${bytecode.length} bytes`);
