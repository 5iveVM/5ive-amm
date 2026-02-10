
pub test_nibble_optimized() -> u64 {
    let a = 100;    // local 0 -> SET_LOCAL_0 (1 byte)
    let b = 200;    // local 1 -> SET_LOCAL_1 (1 byte)  
    let c = 300;    // local 2 -> SET_LOCAL_2 (1 byte)
    let d = 400;    // local 3 -> SET_LOCAL_3 (1 byte)
    
    // Heavy use of locals 0-3 (nibble immediate GET_LOCAL_0-3, 1 byte each)
    let sum1 = a + b;       // GET_LOCAL_0 + GET_LOCAL_1 (2 bytes total)
    let sum2 = c + d;       // GET_LOCAL_2 + GET_LOCAL_3 (2 bytes total)
    let final_result = sum1 + sum2; // GET_LOCAL_4 + GET_LOCAL_5 (varint, 4 bytes total)
    
    return final_result;
}