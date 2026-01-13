import { Buffer } from 'buffer';

const bytecodeLength = 432;

// Method 1: My current method
const lengthBuffer = Buffer.allocUnsafe(4);
const lengthView = new DataView(lengthBuffer.buffer);
lengthView.setUint32(0, bytecodeLength, true);

console.log('Method 1 (DataView):');
console.log('  Buffer bytes:', Array.from(lengthBuffer).map(b => '0x' + b.toString(16).padStart(2, '0')).join(' '));

// Method 2: Direct Buffer write
const lengthBuffer2 = Buffer.allocUnsafe(4);
lengthBuffer2.writeUInt32LE(bytecodeLength, 0);

console.log('\nMethod 2 (writeUInt32LE):');
console.log('  Buffer bytes:', Array.from(lengthBuffer2).map(b => '0x' + b.toString(16).padStart(2, '0')).join(' '));

// Method 3: Manual byte packing
const bytes = [
  bytecodeLength & 0xFF,
  (bytecodeLength >> 8) & 0xFF,
  (bytecodeLength >> 16) & 0xFF,
  (bytecodeLength >> 24) & 0xFF,
];
const lengthBuffer3 = Buffer.from(bytes);

console.log('\nMethod 3 (manual bytes):');
console.log('  Buffer bytes:', Array.from(lengthBuffer3).map(b => '0x' + b.toString(16).padStart(2, '0')).join(' '));

// Test in deploy instruction
const deployData = Buffer.concat([
  Buffer.from([8]), // discriminator
  lengthBuffer2,     // use method 2 (correct)
  Buffer.from([0]),  // permissions
]);

console.log('\nDeploy instruction first 10 bytes:', Array.from(deployData.slice(0, 10)).map(b => '0x' + b.toString(16).padStart(2, '0')).join(' '));
console.log('Bytes 1-4 (length field):', Array.from(deployData.slice(1, 5)));
