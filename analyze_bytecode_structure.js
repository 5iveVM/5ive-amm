#!/usr/bin/env node

const fs = require('fs');

function analyzeArtifact(filepath) {
    const content = JSON.parse(fs.readFileSync(filepath, 'utf-8'));
    const bytecode = Buffer.from(content.bytecode, 'base64');
    return bytecode;
}

function main() {
    const baseline = analyzeArtifact('five-templates/token/build/five-token-template-baseline.five');
    const optimized = analyzeArtifact('five-templates/token/build/five-token-registers.five');

    console.log('📋 Five Bytecode Structure Analysis');
    console.log('='.repeat(80));

    // Header Analysis (First 12 bytes typical for Five)
    console.log('\n📍 Header (first 64 bytes):');
    console.log('Baseline:');
    console.log('  ' + baseline.slice(0, 64).toString('hex'));
    console.log('\nOptimized:');
    console.log('  ' + optimized.slice(0, 64).toString('hex'));

    // Parse Five header
    const magic_baseline = baseline.slice(0, 4).toString('ascii');
    const magic_optimized = optimized.slice(0, 4).toString('ascii');

    console.log(`\nMagic (bytes 0-3): "${magic_baseline}" vs "${magic_optimized}"`);

    if (baseline.length > 4) {
        const flags_baseline = baseline[4];
        const flags_optimized = optimized[4];
        console.log(`Flags (byte 4): 0x${flags_baseline.toString(16)} vs 0x${flags_optimized.toString(16)}`);
    }

    if (baseline.length > 5) {
        const pubcount_baseline = baseline[5];
        const pubcount_optimized = optimized[5];
        console.log(`Public count (byte 5): ${pubcount_baseline} vs ${pubcount_optimized}`);
    }

    if (baseline.length > 6) {
        const totalcount_baseline = baseline[6];
        const totalcount_optimized = optimized[6];
        console.log(`Total count (byte 6): ${totalcount_baseline} vs ${totalcount_optimized}`);
    }

    // Find where the metadata ends and bytecode begins
    // Function names start around offset 7 in typical Five format
    console.log('\n📝 Function Names Section (offset 7-180):');
    const namesSection_baseline = baseline.slice(7, 180);
    const namesSection_optimized = optimized.slice(7, 180);

    console.log('Baseline names section:');
    let asciiBaseline = '';
    for (let i = 0; i < namesSection_baseline.length; i++) {
        const byte = namesSection_baseline[i];
        if (byte >= 32 && byte < 127) {
            asciiBaseline += String.fromCharCode(byte);
        } else if (byte === 0) {
            asciiBaseline += '|';
        } else {
            asciiBaseline += '.';
        }
    }
    console.log('  ' + asciiBaseline);
    console.log('  Hex: ' + namesSection_baseline.toString('hex'));

    console.log('\nOptimized names section:');
    let asciiOptimized = '';
    for (let i = 0; i < namesSection_optimized.length; i++) {
        const byte = namesSection_optimized[i];
        if (byte >= 32 && byte < 127) {
            asciiOptimized += String.fromCharCode(byte);
        } else if (byte === 0) {
            asciiOptimized += '|';
        } else {
            asciiOptimized += '.';
        }
    }
    console.log('  ' + asciiOptimized);
    console.log('  Hex: ' + namesSection_optimized.toString('hex'));

    // Check for padding/separator
    console.log('\n🔍 Transition to bytecode (offsets 180-220):');
    console.log('Baseline: ' + baseline.slice(180, 220).toString('hex'));
    console.log('Optimized: ' + optimized.slice(180, 220).toString('hex'));

    // Look for the dispatcher/function table
    console.log('\n📊 Function Dispatch Area (offsets 0xbd-0x160):');
    console.log('Baseline (0xbd-0x160):');
    console.log('  ' + baseline.slice(0xbd, 0x160).toString('hex'));
    console.log('\nOptimized (0xbd-0x160):');
    console.log('  ' + optimized.slice(0xbd, 0x160).toString('hex'));

    // Find repeated patterns
    console.log('\n🔎 Pattern Analysis - Looking for 0x95 (CALL_REG pattern):');
    let count95baseline = 0;
    let count95optimized = 0;
    for (let i = 0; i < baseline.length; i++) {
        if (baseline[i] === 0x95) count95baseline++;
    }
    for (let i = 0; i < optimized.length; i++) {
        if (optimized[i] === 0x95) count95optimized++;
    }
    console.log(`Baseline: ${count95baseline} occurrences of 0x95 (CALL_REG)`);
    console.log(`Optimized: ${count95optimized} occurrences of 0x95 (CALL_REG)`);

    // Show bytes around "dc 19" pattern (LOAD_PARAM_0 + PUSH_U16 pattern)
    console.log('\n📍 Dispatcher Pattern (0xdc 0x19 = LOAD_PARAM_0 PUSH_U16):');
    let match = 0;
    for (let i = 0; i < baseline.length - 1; i++) {
        if (baseline[i] === 0xdc && baseline[i+1] === 0x19) {
            console.log(`Baseline at 0x${i.toString(16)}: ${baseline.slice(i, i+20).toString('hex')}`);
            match++;
            if (match >= 3) break;
        }
    }

    match = 0;
    for (let i = 0; i < optimized.length - 1; i++) {
        if (optimized[i] === 0xdc && optimized[i+1] === 0x19) {
            console.log(`Optimized at 0x${i.toString(16)}: ${optimized.slice(i, i+20).toString('hex')}`);
            match++;
            if (match >= 3) break;
        }
    }

    // Statistical comparison
    console.log('\n📈 Overall Statistics:');
    console.log(`Baseline size: ${baseline.length} bytes`);
    console.log(`Optimized size: ${optimized.length} bytes`);
    console.log(`Difference: ${baseline.length - optimized.length} bytes`);
    console.log(`Reduction: ${(((baseline.length - optimized.length) / baseline.length) * 100).toFixed(2)}%`);

    // Check if it's just the metadata/function table that differs
    const headerSize = 189; // Where bytecode instructions start based on disassembler
    console.log(`\nEstimated header/metadata size: ~${headerSize} bytes`);
    console.log(`Baseline code (after header): ${baseline.length - headerSize} bytes`);
    console.log(`Optimized code (after header): ${optimized.length - headerSize} bytes`);
    console.log(`Code section difference: ${baseline.length - headerSize - (optimized.length - headerSize)} bytes`);
}

main();
