import fs from 'fs';

try {
    const buffer = fs.readFileSync('counter_debug.bin');
    console.log('Total size:', buffer.length);
    const start = 90;
    const end = 110;
    console.log(`Bytes ${start}-${end}:`);
    for (let i = start; i < end; i++) {
        if (i < buffer.length) {
            console.log(`${i}: 0x${buffer[i].toString(16).padStart(2, '0')}`);
        }
    }
} catch (e) {
    console.error('Error reading file:', e);
}
