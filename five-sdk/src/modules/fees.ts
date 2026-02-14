import { FeeInformation, FIVE_VM_PROGRAM_ID } from "../types.js";
import { RentCalculator } from "../crypto/index.js";
import { getVMState } from "./vm-state.js";

export async function getFees(connection: any, fiveVMProgramId?: string): Promise<{
  deployFeeBps: number;
  executeFeeBps: number;
  adminAccount: string | null;
  feeRecipientAccount: string | null;
}> {
  try {
    const state = await getVMState(connection, fiveVMProgramId);
    return {
      deployFeeBps: state.deployFeeBps,
      executeFeeBps: state.executeFeeBps,
      adminAccount: state.authority,
      feeRecipientAccount: state.feeRecipient
    };
  } catch (error) {
    return {
      deployFeeBps: 0,
      executeFeeBps: 0,
      adminAccount: null,
      feeRecipientAccount: null
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
    const deployFeeBps = vmState.deployFeeBps;

    const feeLamports = Math.floor((rentLamports * deployFeeBps) / 10000);

    return {
      feeBps: deployFeeBps,
      basisLamports: rentLamports,
      feeLamports,
      totalEstimatedCost: rentLamports + feeLamports,
      costBreakdown: {
        basis: RentCalculator.formatSOL(rentLamports),
        fee: RentCalculator.formatSOL(feeLamports),
        total: RentCalculator.formatSOL(rentLamports + feeLamports),
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
    const executeFeeBps = vmState.executeFeeBps;

    const feeLamports = Math.floor((STANDARD_TX_FEE * executeFeeBps) / 10000);

    return {
      feeBps: executeFeeBps,
      basisLamports: STANDARD_TX_FEE,
      feeLamports,
      totalEstimatedCost: STANDARD_TX_FEE + feeLamports,
      costBreakdown: {
        basis: RentCalculator.formatSOL(STANDARD_TX_FEE),
        fee: RentCalculator.formatSOL(feeLamports),
        total: RentCalculator.formatSOL(STANDARD_TX_FEE + feeLamports),
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
  feeRecipientAccount: string | null;
  feesEnabled: boolean;
}> {
  try {
    const [deployFee, executeFee, vmState] = await Promise.all([
      calculateDeployFee(bytecodeSize, connection, fiveVMProgramId),
      calculateExecuteFee(connection, fiveVMProgramId),
      getVMState(connection, fiveVMProgramId),
    ]);

    const feesEnabled = vmState.deployFeeBps > 0 || vmState.executeFeeBps > 0;

    return {
      deploy: deployFee,
      execute: executeFee,
      adminAccount: vmState.authority,
      feeRecipientAccount: vmState.feeRecipient,
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
      feeRecipientAccount: null,
      feesEnabled: false,
    };
  }
}
