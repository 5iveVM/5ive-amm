/**
 * Auto-generated types for Program program
 * Generated from ABI
 */

export interface ProgramProgram {
  /**
   * Call init_pool()
   */
  init_pool(params: Init_poolParams): FunctionBuilder;
}

export interface Init_poolParams {
  accounts: {
    pool: string | { toBase58(): string };
    creator: string | { toBase58(): string };
  };
  args: {
    token_a_mint: string | { toBase58(): string };
    token_b_mint: string | { toBase58(): string };
    token_a_vault: string | { toBase58(): string };
    token_b_vault: string | { toBase58(): string };
    lp_mint: string | { toBase58(): string };
    fee_numerator: number | bigint;
    fee_denominator: number | bigint;
  };
}
