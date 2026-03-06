/**
 * Auto-generated types for Program program
 * Generated from ABI
 */

export interface ProgramProgram {
  /**
   * Call init_pool()
   */
  init_pool(params: Init_poolParams): FunctionBuilder;
  /**
   * Call add_liquidity()
   */
  add_liquidity(params: Add_liquidityParams): FunctionBuilder;
}

export interface Init_poolParams {
  accounts: {
    pool: string | { toBase58(): string };
    creator: string | { toBase58(): string };
  };
}

export interface Add_liquidityParams {
  accounts: {
    pool: string | { toBase58(): string };
    user_token_a: string | { toBase58(): string };
    pool_token_a_vault: string | { toBase58(): string };
    user_authority: string | { toBase58(): string };
  };
  args: {
    amount_a: number | bigint;
    amount_b: number | bigint;
  };
}
