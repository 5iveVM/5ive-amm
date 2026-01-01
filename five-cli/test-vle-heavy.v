
pub test_vle_heavy() -> u64 {
    let unused0 = 0;   // local 0 (nibble immediate, but not used)
    let unused1 = 0;   // local 1 (nibble immediate, but not used) 
    let unused2 = 0;   // local 2 (nibble immediate, but not used)
    let unused3 = 0;   // local 3 (nibble immediate, but not used)
    
    let a = 100;       // local 4 -> SET_LOCAL 4 (2 bytes: opcode + index)
    let b = 200;       // local 5 -> SET_LOCAL 5 (2 bytes: opcode + index)  
    let c = 300;       // local 6 -> SET_LOCAL 6 (2 bytes: opcode + index)
    let d = 400;       // local 7 -> SET_LOCAL 7 (2 bytes: opcode + index)
    
    // Heavy use of locals 4-7 (VLE-based GET_LOCAL + index, 2 bytes each)
    let sum1 = a + b;       // GET_LOCAL 4 + GET_LOCAL 5 (4 bytes total)
    let sum2 = c + d;       // GET_LOCAL 6 + GET_LOCAL 7 (4 bytes total)
    let final_result = sum1 + sum2; // GET_LOCAL 8 + GET_LOCAL 9 (4 bytes total)
    
    return final_result;
}