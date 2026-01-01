// Large script that forces VLE usage by using high-index locals
// Same logic as large-nibble.v but forces locals 4+ to avoid nibble optimization

pub main_calculation(input1: u64, input2: u64) -> u64 {
    // Skip nibble immediate slots (locals 0-3)
    let unused0 = 0;        // local 0 (nibble slot wasted)
    let unused1 = 0;        // local 1 (nibble slot wasted)
    let unused2 = 0;        // local 2 (nibble slot wasted)
    let unused3 = 0;        // local 3 (nibble slot wasted)
    
    // Use locals 4+ exclusively (all VLE, 2 bytes each)
    let primary = input1;       // local 4 -> SET_LOCAL 4 (2 bytes)
    let secondary = input2;     // local 5 -> SET_LOCAL 5 (2 bytes)
    let multiplier = 10;        // local 6 -> SET_LOCAL 6 (2 bytes)
    let accumulator = 0;        // local 7 -> SET_LOCAL 7 (2 bytes)
    
    // Many operations using VLE (2 bytes each access)
    let step1 = primary * multiplier;           // GET_LOCAL 4 * GET_LOCAL 6 (4 bytes)
    let step2 = secondary * multiplier;         // GET_LOCAL 5 * GET_LOCAL 6 (4 bytes)
    let step3 = primary + secondary;            // GET_LOCAL 4 + GET_LOCAL 5 (4 bytes)
    let step4 = step1 + accumulator;            // step1 + GET_LOCAL 7 (2 bytes)
    let step5 = step2 + accumulator;            // step2 + GET_LOCAL 7 (2 bytes)
    let step6 = step3 * multiplier;             // step3 * GET_LOCAL 6 (2 bytes)
    let step7 = primary + step4;                // GET_LOCAL 4 + step4 (2 bytes)
    let step8 = secondary + step5;              // GET_LOCAL 5 + step5 (2 bytes)
    let step9 = multiplier + step6;             // GET_LOCAL 6 + step6 (2 bytes)
    let final_result = step7 + step8 + step9;
    
    return final_result;
}