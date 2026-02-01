#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

function analyzeArtifact(filepath) {
    const content = JSON.parse(fs.readFileSync(filepath, 'utf-8'));
    const bytecode = Buffer.from(content.bytecode, 'base64');
    return bytecode;
}

function findFirstDifference(baseline, optimized) {
    const minLen = Math.min(baseline.length, optimized.length);
    for (let i = 0; i < minLen; i++) {
        if (baseline[i] !== optimized[i]) {
            return i;
        }
    }
    return minLen === baseline.length && minLen === optimized.length ? -1 : minLen;
}

function dumpBytecodeAround(buffer, offset, context = 32) {
    const start = Math.max(0, offset - context);
    const end = Math.min(buffer.length, offset + context);

    console.log(`\nBytecode around offset 0x${offset.toString(16)} (${offset}):`);
    console.log('='.repeat(80));

    for (let i = start; i < end; i += 16) {
        const lineEnd = Math.min(i + 16, end);
        const hex = buffer.slice(i, lineEnd).toString('hex');
        const ascii = buffer.slice(i, lineEnd).map(b =>
            b >= 32 && b < 127 ? String.fromCharCode(b) : '.'
        ).join('');

        const marker = offset >= i && offset < lineEnd ? ' <--- DIFFERENCE HERE' : '';
        const paddedHex = hex.padEnd(32, ' ');
        console.log(`${('00000' + i.toString(16)).slice(-6)}: ${paddedHex} ${ascii}${marker}`);
    }
}

function decodeVLE(buffer, offset) {
    let value = 0;
    let shift = 0;
    let pos = offset;
    let bytes = 0;

    while (pos < buffer.length && bytes < 4) {
        const byte = buffer[pos];
        value |= (byte & 0x7F) << shift;
        bytes++;

        if (!(byte & 0x80)) break;

        shift += 7;
        pos++;
    }

    return { value, bytes, pos };
}

function analyzeInstruction(buffer, offset) {
    if (offset >= buffer.length) return null;

    const opcode = buffer[offset];
    let instr = { offset, opcode, bytes: 1 };

    // Check for JUMP opcodes
    if (opcode === 0x01) { // JUMP
        instr.name = 'JUMP';
        if (offset + 2 < buffer.length) {
            const target = buffer.readUInt16LE(offset + 1);
            instr.target = target;
            instr.bytes = 3;
        }
    } else if (opcode === 0x02) { // JUMP_IF
        instr.name = 'JUMP_IF';
        if (offset + 2 < buffer.length) {
            const target = buffer.readUInt16LE(offset + 1);
            instr.target = target;
            instr.bytes = 3;
        }
    } else if (opcode === 0x03) { // JUMP_IF_NOT
        instr.name = 'JUMP_IF_NOT';
        if (offset + 2 < buffer.length) {
            const target = buffer.readUInt16LE(offset + 1);
            instr.target = target;
            instr.bytes = 3;
        }
    } else {
        instr.name = `OPCODE_0x${opcode.toString(16).padStart(2, '0')}`;
    }

    return instr;
}

function main() {
    const baselineFile = 'five-templates/token/build/five-token-template-baseline.five';
    const optimizedFile = 'five-templates/token/build/five-token-registers.five';

    if (!fs.existsSync(baselineFile) || !fs.existsSync(optimizedFile)) {
        console.error('❌ Artifact files not found');
        console.error(`  Baseline: ${baselineFile}`);
        console.error(`  Optimized: ${optimizedFile}`);
        process.exit(1);
    }

    const baseline = analyzeArtifact(baselineFile);
    const optimized = analyzeArtifact(optimizedFile);

    console.log('📊 Bytecode Comparison');
    console.log('='.repeat(80));
    console.log(`Baseline size:  ${baseline.length} bytes (0x${baseline.length.toString(16)})`);
    console.log(`Optimized size: ${optimized.length} bytes (0x${optimized.length.toString(16)})`);
    console.log(`Difference:     ${baseline.length - optimized.length} bytes`);

    const firstDiff = findFirstDifference(baseline, optimized);
    console.log(`\nFirst difference at: 0x${firstDiff.toString(16)} (${firstDiff})`);

    // Show bytes around first difference
    dumpBytecodeAround(baseline, firstDiff, 20);
    console.log('\nOptimized:');
    dumpBytecodeAround(optimized, firstDiff, 20);

    // Analyze instructions around the difference
    console.log('\n📝 Instruction Analysis');
    console.log('='.repeat(80));

    // Find all JUMP instructions in both versions
    const jumpsBaseline = [];
    const jumpsOptimized = [];

    for (let i = 0; i < baseline.length - 2; i++) {
        const instr = analyzeInstruction(baseline, i);
        if (instr && instr.target !== undefined) {
            jumpsBaseline.push(instr);
        }
    }

    for (let i = 0; i < optimized.length - 2; i++) {
        const instr = analyzeInstruction(optimized, i);
        if (instr && instr.target !== undefined) {
            jumpsOptimized.push(instr);
        }
    }

    console.log(`\nBaseline JUMP instructions: ${jumpsBaseline.length}`);
    jumpsBaseline.forEach(j => {
        const outOfBounds = j.target >= baseline.length;
        const marker = outOfBounds ? '⚠️  OUT OF BOUNDS' : '✓';
        console.log(`  ${marker} 0x${j.offset.toString(16).padStart(4, '0')}: ${j.name} 0x${j.target.toString(16).padStart(4, '0')}`);
    });

    console.log(`\nOptimized JUMP instructions: ${jumpsOptimized.length}`);
    jumpsOptimized.forEach(j => {
        const outOfBounds = j.target >= optimized.length;
        const marker = outOfBounds ? '❌ OUT OF BOUNDS' : '✓';
        console.log(`  ${marker} 0x${j.offset.toString(16).padStart(4, '0')}: ${j.name} 0x${j.target.toString(16).padStart(4, '0')}`);
    });

    // Check for problematic jumps
    const problemJumps = jumpsOptimized.filter(j => j.target >= optimized.length);
    if (problemJumps.length > 0) {
        console.log(`\n❌ Found ${problemJumps.length} out-of-bounds JUMP instructions in optimized bytecode!`);
        console.log('\nThis confirms the compiler bug: JUMP targets exceed bytecode length');

        problemJumps.forEach(j => {
            console.log(`\n  Problem JUMP at 0x${j.offset.toString(16)}:`);
            console.log(`    Opcode: 0x${j.opcode.toString(16).padStart(2, '0')} (${j.name})`);
            console.log(`    Target: 0x${j.target.toString(16)} (${j.target})`);
            console.log(`    Bytecode length: ${optimized.length}`);
            console.log(`    Out of bounds by: ${j.target - optimized.length + 1} bytes`);
        });
    }
}

main();
