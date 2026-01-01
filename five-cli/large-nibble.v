// Large script optimized for nibble immediate opcodes
// Focuses on extensive use of locals 0-3 with many operations

pub main_calculation(input1: u64, input2: u64) -> u64 {
    // Use locals 0-3 extensively (nibble immediate optimization)
    let primary = input1;       // local 0 -> SET_LOCAL_0 (1 byte)
    let secondary = input2;     // local 1 -> SET_LOCAL_1 (1 byte)
    let multiplier = 10;        // local 2 -> SET_LOCAL_2 (1 byte)
    let accumulator = 0;        // local 3 -> SET_LOCAL_3 (1 byte)
    
    // Many operations using nibble immediates (1 byte each access)
    let step1 = primary * multiplier;           // GET_LOCAL_0 * GET_LOCAL_2 (2 bytes)
    let step2 = secondary * multiplier;         // GET_LOCAL_1 * GET_LOCAL_2 (2 bytes)
    let step3 = primary + secondary;            // GET_LOCAL_0 + GET_LOCAL_1 (2 bytes)
    let step4 = step1 + accumulator;            // step1 + GET_LOCAL_3 (1 byte)
    let step5 = step2 + accumulator;            // step2 + GET_LOCAL_3 (1 byte)
    let step6 = step3 * multiplier;             // step3 * GET_LOCAL_2 (1 byte)
    let step7 = primary + step4;                // GET_LOCAL_0 + step4 (1 byte)
    let step8 = secondary + step5;              // GET_LOCAL_1 + step5 (1 byte)
    let step9 = multiplier + step6;             // GET_LOCAL_2 + step6 (1 byte)
    let final_result = step7 + step8 + step9;
    
    return final_result;
}