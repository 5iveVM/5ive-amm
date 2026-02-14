import { FeeInformation, FIVE_VM_PROGRAM_ID } from "../types.js";
import { RentCalculator } from "../crypto/index.js";
import { getVMState } from "./vm-state.js";

export async function getFees(connection: any, fiveVMProgramId?: string): Promise<{
  deployFeeLamports: number;
  executeFeeLamports: number;
  // Backward-compatible aliases; deprecated.
  deployFeeBps: number;
  executeFeeBps: number;
  adminAccount: string | null;
}> {
  try {
    const state = await getVMState(connection, fiveVMProgramId);
    return {
      deployFeeLamports: state.deployFeeLamports,
      executeFeeLamports: state.executeFeeLamports,
      deployFeeBps: state.deployFeeLamports,
      executeFeeBps: state.executeFeeLamports,
      adminAccount: state.authority,
    };
  } catch (error) {
    return {
      deployFeeLamports: 0,
      executeFeeLamports: 0,
      deployFeeBps: 0,
      executeFeeBps: 0,
      adminAccount: null,
    };
  }
}

export async function calculateDeployFee(
  bytecodeSize: number,
  connection?: any,
  fiveVMProgramId?: string,
): Promise<FeeInformation> {
  try {
    const accountSize = 64 + bytecodeSize;
    const rentLamports = await RentCalculator.calculateRentExemption(accountSize);

    const vmState = await getVMState(connection, fiveVMProgramId);
    const deployFeeLamports = vmState.deployFeeLamports;

    return {
      feeBps: 0,
      basisLamports: rentLamports,
      feeLamports: deployFeeLamports,
      totalEstimatedCost: rentLamports + deployFeeLamports,
      costBreakdown: {
        basis: RentCalculator.formatSOL(rentLamports),
        fee: RentCalculator.formatSOL(deployFeeLamports),
        total: RentCalculator.formatSOL(rentLamports + deployFeeLamports),
      },
    };
  } catch (error) {
    const accountSize = 64 + bytecodeSize;
    const rentLamports = await RentCalculator.calculateRentExemption(accountSize);

    return {
      feeBps: 0,
      basisLamports: rentLamports,
      feeLamports: 0,
      totalEstimatedCost: rentLamports,
      costBreakdown: {
        basis: RentCalculator.formatSOL(rentLamports),
        fee: "0 SOL",
        total: RentCalculator.formatSOL(rentLamports),
      },
    };
  }
}

export async function calculateExecuteFee(
  connection?: any,
  fiveVMProgramId?: string,
): Promise<FeeInformation> {
  const STANDARD_TX_FEE = 5000;

  try {
    const vmState = await getVMState(connection, fiveVMProgramId);
    const executeFeeLamports = vmState.executeFeeLamports;

    return {
      feeBps: 0,
      basisLamports: STANDARD_TX_FEE,
      feeLamports: executeFeeLamports,
      totalEstimatedCost: STANDARD_TX_FEE + executeFeeLamports,
      costBreakdown: {
        basis: RentCalculator.formatSOL(STANDARD_TX_FEE),
        fee: RentCalculator.formatSOL(executeFeeLamports),
        total: RentCalculator.formatSOL(STANDARD_TX_FEE + executeFeeLamports),
      },
    };
  } catch (error) {
    return {
      feeBps: 0,
      basisLamports: 5000,
      feeLamports: 0,
      totalEstimatedCost: 5000,
      costBreakdown: {
        basis: "0.000005 SOL",
        fee: "0 SOL",
        total: "0.000005 SOL",
      },
    };
  }
}

export async function getFeeInformation(
  bytecodeSize: number,
  connection?: any,
  fiveVMProgramId?: string,
): Promise<{
  deploy: FeeInformation;
  execute: FeeInformation;
  adminAccount: string | null;
  feesEnabled: boolean;
}> {
  try {
    const [deployFee, executeFee, vmState] = await Promise.all([
      calculateDeployFee(bytecodeSize, connection, fiveVMProgramId),
      calculateExecuteFee(connection, fiveVMProgramId),
      getVMState(connection, fiveVMProgramId),
    ]);

    const feesEnabled = vmState.deployFeeLamports > 0 || vmState.executeFeeLamports > 0;

    return {
      deploy: deployFee,
      execute: executeFee,
      adminAccount: vmState.authority,
      feesEnabled,
    };
  } catch (error) {
    const deployFee = await calculateDeployFee(
      bytecodeSize,
      connection,
      fiveVMProgramId,
    );
    const executeFee = await calculateExecuteFee(connection, fiveVMProgramId);

    return {
      deploy: deployFee,
      execute: executeFee,
      adminAccount: null,
      feesEnabled: false,
    };
  }
}
