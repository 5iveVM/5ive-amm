import fs from 'fs';

try {
    const json = JSON.parse(fs.readFileSync('build/five-counter-template.five', 'utf-8'));
    // Assuming bytecode is hex string in 'bytecode' field or similar
    // Check structure first? Usually it has 'bytecode'.
    // If not, print keys.
    if (!json.bytecode) {
        console.log('Keys:', Object.keys(json));
        console.log('JSON:', JSON.stringify(json, null, 2).substring(0, 500));
    } else {
        const bytecodeBase64 = json.bytecode;
        const buffer = Buffer.from(bytecodeBase64, 'base64');
        console.log('Total size:', buffer.length);
        const start = 90;
        const end = 110;
        console.log(`Bytes ${start}-${end}:`);
        for (let i = start; i < end; i++) {
            if (i < buffer.length) {
                console.log(`${i}: 0x${buffer[i].toString(16).padStart(2, '0')}`);
            }
        }
    }
} catch (e) {
    console.error('Error reading file:', e);
}
