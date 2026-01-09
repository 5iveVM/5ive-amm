
import fs from 'fs';
import path from 'path';

const binPath = 'src/counter.bin';
const abiPath = 'src/counter.abi.json';
const outPath = 'build/five-counter-template.five';

if (!fs.existsSync(binPath)) {
    console.error(`Error: ${binPath} not found`);
    process.exit(1);
}
if (!fs.existsSync(abiPath)) {
    console.error(`Error: ${abiPath} not found`);
    process.exit(1);
}

const bytecode = fs.readFileSync(binPath);
const abi = JSON.parse(fs.readFileSync(abiPath, 'utf-8'));

const artifact = {
    bytecode: bytecode.toString('base64'),
    abi: abi
};

if (!fs.existsSync('build')) {
    fs.mkdirSync('build');
}

fs.writeFileSync(outPath, JSON.stringify(artifact, null, 2));
console.log(`Created ${outPath}`);
