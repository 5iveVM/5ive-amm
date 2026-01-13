/**
 * Auto-generated types for Program program
 * Generated from ABI
 */

export interface ProgramProgram {
  /**
   * Call initialize()
   */
  initialize(params: InitializeParams): FunctionBuilder;
  /**
   * Call increment()
   */
  increment(params: IncrementParams): FunctionBuilder;
  /**
   * Call decrement()
   */
  decrement(params: DecrementParams): FunctionBuilder;
  /**
   * Call add_amount()
   */
  add_amount(params: Add_amountParams): FunctionBuilder;
  /**
   * Call get_count()
   */
  get_count(params: Get_countParams): FunctionBuilder;
  /**
   * Call reset()
   */
  reset(params: ResetParams): FunctionBuilder;
}

export interface InitializeParams {
  accounts: {
    counter: string | { toBase58(): string };
    owner: string | { toBase58(): string };
  };
}

export interface IncrementParams {
  accounts: {
    counter: string | { toBase58(): string };
    owner: string | { toBase58(): string };
  };
}

export interface DecrementParams {
  accounts: {
    counter: string | { toBase58(): string };
    owner: string | { toBase58(): string };
  };
}

export interface Add_amountParams {
  accounts: {
    counter: string | { toBase58(): string };
    owner: string | { toBase58(): string };
  };
  args: {
    amount: number | bigint;
  };
}

export interface Get_countParams {
  accounts: {
    counter: string | { toBase58(): string };
  };
}

export interface ResetParams {
  accounts: {
    counter: string | { toBase58(): string };
    owner: string | { toBase58(): string };
  };
}
