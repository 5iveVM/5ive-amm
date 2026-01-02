import fs from 'fs';
import path from 'path';

const binPath = 'src/counter.fbin';
const abiPath = 'src/counter.abi.json';
const outPath = 'build/five-counter-template.five';

if (!fs.existsSync(binPath) || !fs.existsSync(abiPath)) {
    console.error("Missing bin or abi file");
    process.exit(1);
}

const bytecode = fs.readFileSync(binPath);
const abi = JSON.parse(fs.readFileSync(abiPath, 'utf8'));

const artifact = {
    name: "five-counter-template",
    bytecode: bytecode.toString('base64'),
    abi: abi
};

if (!fs.existsSync('build')) {
    fs.mkdirSync('build');
}

fs.writeFileSync(outPath, JSON.stringify(artifact, null, 2));
console.log(`Artifact created at ${outPath}`);
