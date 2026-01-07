import * as wasm from "./five_vm_wasm_bg.wasm";
export * from "./five_vm_wasm_bg.js";
import { __wbg_set_wasm } from "./five_vm_wasm_bg.js";
__wbg_set_wasm(wasm);
wasm.__wbindgen_start();
