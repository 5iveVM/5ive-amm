import { __deployTestUtils } from "../../modules/deploy.js";

describe("deploy wire format builders", () => {
  it("encodes initialize vm state with discriminator and bump", () => {
    const data = __deployTestUtils.buildInitializeVmStateInstructionData(0);
    expect(Array.from(data)).toEqual([0, 0]);
  });

  it("encodes init-large payload with expected size and optional first chunk", () => {
    const chunk = Uint8Array.from([1, 2, 3]);
    const data = __deployTestUtils.createInitLargeProgramInstructionData(513, chunk);
    expect(data[0]).toBe(4);
    expect(data.readUInt32LE(1)).toBe(513);
    expect(Array.from(data.slice(5))).toEqual([1, 2, 3]);
  });

  it("encodes append and finalize discriminators", () => {
    const append = __deployTestUtils.createAppendBytecodeInstructionData(
      Uint8Array.from([9, 8]),
    );
    const finalize = __deployTestUtils.createFinalizeScriptInstructionData();
    expect(Array.from(append)).toEqual([5, 9, 8]);
    expect(Array.from(finalize)).toEqual([7]);
  });
});
