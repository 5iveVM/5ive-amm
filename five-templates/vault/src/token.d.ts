/**
 * Auto-generated types for Program program
 * Generated from ABI
 */

export interface ProgramProgram {
  /**
   * Call init_mint()
   */
  init_mint(params: Init_mintParams): FunctionBuilder;
  /**
   * Call init_token_account()
   */
  init_token_account(params: Init_token_accountParams): FunctionBuilder;
  /**
   * Call mint()
   */
  mint(params: MintParams): FunctionBuilder;
  /**
   * Call burn()
   */
  burn(params: BurnParams): FunctionBuilder;
  /**
   * Call transfer()
   */
  transfer(params: TransferParams): FunctionBuilder;
  /**
   * Call approve()
   */
  approve(params: ApproveParams): FunctionBuilder;
  /**
   * Call revoke()
   */
  revoke(params: RevokeParams): FunctionBuilder;
  /**
   * Call transfer_approved()
   */
  transfer_approved(params: Transfer_approvedParams): FunctionBuilder;
  /**
   * Call freeze_account()
   */
  freeze_account(params: Freeze_accountParams): FunctionBuilder;
  /**
   * Call thaw_account()
   */
  thaw_account(params: Thaw_accountParams): FunctionBuilder;
  /**
   * Call close_account()
   */
  close_account(params: Close_accountParams): FunctionBuilder;
  /**
   * Call set_mint_authority()
   */
  set_mint_authority(params: Set_mint_authorityParams): FunctionBuilder;
}

export interface Init_mintParams {
  accounts: {
    mint: string | { toBase58(): string };
    authority: string | { toBase58(): string };
  };
  args: {
    decimals: number;
  };
}

export interface Init_token_accountParams {
  accounts: {
    token_account: string | { toBase58(): string };
    owner: string | { toBase58(): string };
    mint: string | { toBase58(): string };
  };
}

export interface MintParams {
  accounts: {
    mint: string | { toBase58(): string };
    token_account: string | { toBase58(): string };
    authority: string | { toBase58(): string };
  };
  args: {
    amount: number | bigint;
  };
}

export interface BurnParams {
  accounts: {
    mint: string | { toBase58(): string };
    token_account: string | { toBase58(): string };
    owner: string | { toBase58(): string };
  };
  args: {
    amount: number | bigint;
  };
}

export interface TransferParams {
  accounts: {
    from_account: string | { toBase58(): string };
    to_account: string | { toBase58(): string };
    owner: string | { toBase58(): string };
  };
  args: {
    amount: number | bigint;
  };
}

export interface ApproveParams {
  accounts: {
    token_account: string | { toBase58(): string };
    owner: string | { toBase58(): string };
  };
  args: {
    delegate: any;
    amount: number | bigint;
  };
}

export interface RevokeParams {
  accounts: {
    token_account: string | { toBase58(): string };
    owner: string | { toBase58(): string };
  };
}

export interface Transfer_approvedParams {
  accounts: {
    from_account: string | { toBase58(): string };
    to_account: string | { toBase58(): string };
    delegate: string | { toBase58(): string };
  };
  args: {
    amount: number | bigint;
  };
}

export interface Freeze_accountParams {
  accounts: {
    mint: string | { toBase58(): string };
    token_account: string | { toBase58(): string };
    authority: string | { toBase58(): string };
  };
}

export interface Thaw_accountParams {
  accounts: {
    mint: string | { toBase58(): string };
    token_account: string | { toBase58(): string };
    authority: string | { toBase58(): string };
  };
}

export interface Close_accountParams {
  accounts: {
    token_account: string | { toBase58(): string };
    owner: string | { toBase58(): string };
  };
}

export interface Set_mint_authorityParams {
  accounts: {
    mint: string | { toBase58(): string };
    authority: string | { toBase58(): string };
  };
  args: {
    new_authority: any;
  };
}
