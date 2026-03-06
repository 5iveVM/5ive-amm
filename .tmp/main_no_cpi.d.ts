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
  /**
   * Call swap()
   */
  swap(params: SwapParams): FunctionBuilder;
  /**
   * Call remove_liquidity()
   */
  remove_liquidity(params: Remove_liquidityParams): FunctionBuilder;
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

export interface Add_liquidityParams {
  accounts: {
    pool: string | { toBase58(): string };
    user_token_a: string | { toBase58(): string };
    user_token_b: string | { toBase58(): string };
    pool_token_a_vault: string | { toBase58(): string };
    pool_token_b_vault: string | { toBase58(): string };
    lp_mint: string | { toBase58(): string };
    user_lp_account: string | { toBase58(): string };
    user_authority: string | { toBase58(): string };
  };
  args: {
    amount_a: number | bigint;
    amount_b: number | bigint;
  };
}

export interface SwapParams {
  accounts: {
    pool: string | { toBase58(): string };
    user_source: string | { toBase58(): string };
    user_destination: string | { toBase58(): string };
    pool_source_vault: string | { toBase58(): string };
    pool_destination_vault: string | { toBase58(): string };
    user_authority: string | { toBase58(): string };
  };
  args: {
    amount_in: number | bigint;
    is_a_to_b: boolean;
  };
}

export interface Remove_liquidityParams {
  accounts: {
    pool: string | { toBase58(): string };
    user_lp_account: string | { toBase58(): string };
    user_token_a: string | { toBase58(): string };
    user_token_b: string | { toBase58(): string };
    pool_token_a_vault: string | { toBase58(): string };
    pool_token_b_vault: string | { toBase58(): string };
    lp_mint: string | { toBase58(): string };
    user_authority: string | { toBase58(): string };
  };
  args: {
    lp_amount: number | bigint;
  };
}
