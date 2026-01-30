const fs = require('fs');

const compiledFile = 'build/five-token-template.five';
const content = fs.readFileSync(compiledFile, 'utf-8');
const parsed = JSON.parse(content);
const bytecodeBase64 = parsed.bytecode;
const bytecodeBuffer = Buffer.from(bytecodeBase64, 'base64');

console.log('====== Bytecode Register Opcode Analysis ======');
console.log(`Bytecode size: ${bytecodeBuffer.length} bytes\n`);

const registerOpcodes = [];
const opcodeNames = {
    0xb0: 'LOAD_REG_U8',
    0xb1: 'LOAD_REG_U32',
    0xb2: 'LOAD_REG_U64',
    0xb3: 'LOAD_REG_BOOL',
    0xb4: 'LOAD_REG_PUBKEY',
    0xb5: 'ADD_REG',
    0xb6: 'SUB_REG',
    0xb7: 'MUL_REG',
    0xb8: 'DIV_REG',
    0xb9: 'EQ_REG',
    0xba: 'GT_REG',
    0xbb: 'LT_REG',
    0xbc: 'PUSH_REG',
    0xbd: 'POP_REG',
    0xbe: 'COPY_REG',
    0xbf: 'CLEAR_REG',
    0xcb: 'LOAD_FIELD_REG',
    0xcc: 'REQUIRE_GTE_REG',
    0xcd: 'STORE_FIELD_REG',
    0xce: 'ADD_FIELD_REG',
    0xcf: 'SUB_FIELD_REG',
};

for (let i = 0; i < bytecodeBuffer.length; i++) {
    const opcode = bytecodeBuffer[i];
    const name = opcodeNames[opcode];
    if (name) {
        registerOpcodes.push({ offset: i, opcode: '0x' + opcode.toString(16).toUpperCase().padStart(2, '0'), name });
    }
}

console.log(`Found ${registerOpcodes.length} register opcodes:\n`);

// Group by type
const loadOps = registerOpcodes.filter(op => op.name.startsWith('LOAD'));
const arithmeticOps = registerOpcodes.filter(op => op.name.match(/ADD_REG|SUB_REG|MUL_REG|DIV_REG/));
const comparisonOps = registerOpcodes.filter(op => op.name.match(/EQ_REG|GT_REG|LT_REG/));
const stackBridgeOps = registerOpcodes.filter(op => op.name.match(/PUSH_REG|POP_REG|COPY_REG|CLEAR_REG/));
const fieldOps = registerOpcodes.filter(op => op.name.match(/FIELD_REG|REQUIRE_GTE_REG/));

if (loadOps.length > 0) {
    console.log(`Load Operations (${loadOps.length}):`);
    loadOps.forEach(op => console.log(`  [${op.offset}] ${op.opcode} ${op.name}`));
    console.log('');
}

if (arithmeticOps.length > 0) {
    console.log(`Arithmetic Operations (${arithmeticOps.length}):`);
    arithmeticOps.forEach(op => console.log(`  [${op.offset}] ${op.opcode} ${op.name}`));
    console.log('');
}

if (comparisonOps.length > 0) {
    console.log(`Comparison Operations (${comparisonOps.length}):`);
    comparisonOps.forEach(op => console.log(`  [${op.offset}] ${op.opcode} ${op.name}`));
    console.log('');
}

if (stackBridgeOps.length > 0) {
    console.log(`Stack Bridge Operations (${stackBridgeOps.length}):`);
    stackBridgeOps.forEach(op => console.log(`  [${op.offset}] ${op.opcode} ${op.name}`));
    console.log('');
}

if (fieldOps.length > 0) {
    console.log(`Field Operations (${fieldOps.length}):`);
    fieldOps.forEach(op => console.log(`  [${op.offset}] ${op.opcode} ${op.name}`));
    console.log('');
}

console.log('====== Summary ======');
if (registerOpcodes.length > 0) {
    console.log(`✓ Register optimizations CONFIRMED`);
    console.log(`✓ Total register opcodes: ${registerOpcodes.length}`);
    console.log(`✓ Bytecode includes optimized register-based operations`);
} else {
    console.log(`✗ No register opcodes found in bytecode`);
}
