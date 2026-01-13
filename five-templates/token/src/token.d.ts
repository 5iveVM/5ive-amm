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
   * Call mint_to()
   */
  mint_to(params: Mint_toParams): FunctionBuilder;
  /**
   * Call transfer()
   */
  transfer(params: TransferParams): FunctionBuilder;
  /**
   * Call transfer_from()
   */
  transfer_from(params: Transfer_fromParams): FunctionBuilder;
  /**
   * Call approve()
   */
  approve(params: ApproveParams): FunctionBuilder;
  /**
   * Call revoke()
   */
  revoke(params: RevokeParams): FunctionBuilder;
  /**
   * Call burn()
   */
  burn(params: BurnParams): FunctionBuilder;
  /**
   * Call freeze_account()
   */
  freeze_account(params: Freeze_accountParams): FunctionBuilder;
  /**
   * Call thaw_account()
   */
  thaw_account(params: Thaw_accountParams): FunctionBuilder;
  /**
   * Call set_mint_authority()
   */
  set_mint_authority(params: Set_mint_authorityParams): FunctionBuilder;
  /**
   * Call set_freeze_authority()
   */
  set_freeze_authority(params: Set_freeze_authorityParams): FunctionBuilder;
  /**
   * Call disable_mint()
   */
  disable_mint(params: Disable_mintParams): FunctionBuilder;
  /**
   * Call disable_freeze()
   */
  disable_freeze(params: Disable_freezeParams): FunctionBuilder;
}

export interface Init_mintParams {
  accounts: {
    mint_account: string | { toBase58(): string };
    authority: string | { toBase58(): string };
  };
  args: {
    freeze_authority: string | { toBase58(): string };
    decimals: number;
    name: string;
    symbol: string;
    uri: string;
  };
}

export interface Init_token_accountParams {
  accounts: {
    token_account: string | { toBase58(): string };
    owner: string | { toBase58(): string };
  };
  args: {
    mint: string | { toBase58(): string };
  };
}

export interface Mint_toParams {
  accounts: {
    mint_state: string | { toBase58(): string };
    destination_account: string | { toBase58(): string };
    mint_authority: string | { toBase58(): string };
  };
  args: {
    amount: number | bigint;
  };
}

export interface TransferParams {
  accounts: {
    source_account: string | { toBase58(): string };
    destination_account: string | { toBase58(): string };
    owner: string | { toBase58(): string };
  };
  args: {
    amount: number | bigint;
  };
}

export interface Transfer_fromParams {
  accounts: {
    source_account: string | { toBase58(): string };
    destination_account: string | { toBase58(): string };
    authority: string | { toBase58(): string };
  };
  args: {
    amount: number | bigint;
  };
}

export interface ApproveParams {
  accounts: {
    source_account: string | { toBase58(): string };
    owner: string | { toBase58(): string };
  };
  args: {
    delegate: string | { toBase58(): string };
    amount: number | bigint;
  };
}

export interface RevokeParams {
  accounts: {
    source_account: string | { toBase58(): string };
    owner: string | { toBase58(): string };
  };
}

export interface BurnParams {
  accounts: {
    mint_state: string | { toBase58(): string };
    source_account: string | { toBase58(): string };
    owner: string | { toBase58(): string };
  };
  args: {
    amount: number | bigint;
  };
}

export interface Freeze_accountParams {
  accounts: {
    mint_state: string | { toBase58(): string };
    account_to_freeze: string | { toBase58(): string };
    freeze_authority: string | { toBase58(): string };
  };
}

export interface Thaw_accountParams {
  accounts: {
    mint_state: string | { toBase58(): string };
    account_to_thaw: string | { toBase58(): string };
    freeze_authority: string | { toBase58(): string };
  };
}

export interface Set_mint_authorityParams {
  accounts: {
    mint_state: string | { toBase58(): string };
    current_authority: string | { toBase58(): string };
  };
  args: {
    new_authority: string | { toBase58(): string };
  };
}

export interface Set_freeze_authorityParams {
  accounts: {
    mint_state: string | { toBase58(): string };
    current_freeze_authority: string | { toBase58(): string };
  };
  args: {
    new_freeze_authority: string | { toBase58(): string };
  };
}

export interface Disable_mintParams {
  accounts: {
    mint_state: string | { toBase58(): string };
    current_authority: string | { toBase58(): string };
  };
}

export interface Disable_freezeParams {
  accounts: {
    mint_state: string | { toBase58(): string };
    current_freeze_authority: string | { toBase58(): string };
  };
}
