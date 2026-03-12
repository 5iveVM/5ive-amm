/**
 * Auto-generated types for Program program
 * Generated from ABI
 */

export interface ProgramProgram {
  /**
   * Call cp_swap_math()
   */
  cp_swap_math(params: Cp_swap_mathParams): FunctionBuilder;
  /**
   * Call accrue_interest_math()
   */
  accrue_interest_math(params: Accrue_interest_mathParams): FunctionBuilder;
  /**
   * Call recursive_sum()
   */
  recursive_sum(params: Recursive_sumParams): FunctionBuilder;
  /**
   * Call recursive_bounded()
   */
  recursive_bounded(params: Recursive_boundedParams): FunctionBuilder;
  /**
   * Call recursive_error()
   */
  recursive_error(params: Recursive_errorParams): FunctionBuilder;
  /**
   * Call stable_swap_invariant_iterative()
   */
  stable_swap_invariant_iterative(params: Stable_swap_invariant_iterativeParams): FunctionBuilder;
  /**
   * Call utilization_kink_rate()
   */
  utilization_kink_rate(params: Utilization_kink_rateParams): FunctionBuilder;
  /**
   * Call funding_rate_path()
   */
  funding_rate_path(params: Funding_rate_pathParams): FunctionBuilder;
  /**
   * Call collateral_health_loop()
   */
  collateral_health_loop(params: Collateral_health_loopParams): FunctionBuilder;
}

export interface Cp_swap_mathParams {
  args: {
    amount_in: number | bigint;
    fee_bps: number | bigint;
  };
}

export interface Accrue_interest_mathParams {
  args: {
    debt: number | bigint;
    rate_bps: number | bigint;
    slots: number | bigint;
  };
}

export interface Recursive_sumParams {
  args: {
    depth: number | bigint;
    acc: number | bigint;
  };
}

export interface Recursive_boundedParams {
  args: {
    depth: number | bigint;
  };
}

export interface Recursive_errorParams {
  args: {
    depth: number | bigint;
  };
}

export interface Stable_swap_invariant_iterativeParams {
  args: {
    reserve_x: number | bigint;
    reserve_y: number | bigint;
    amp_factor: number | bigint;
    iterations: number | bigint;
  };
}

export interface Utilization_kink_rateParams {
  args: {
    util_bps: number | bigint;
    base_rate_bps: number | bigint;
    slope_low_bps: number | bigint;
    slope_high_bps: number | bigint;
    kink_bps: number | bigint;
  };
}

export interface Funding_rate_pathParams {
  args: {
    skew_bps: number | bigint;
    base_rate_bps: number | bigint;
    velocity_bps: number | bigint;
    intervals: number | bigint;
  };
}

export interface Collateral_health_loopParams {
  args: {
    collateral: number | bigint;
    debt: number | bigint;
    liquidation_threshold_bps: number | bigint;
    haircut_bps: number | bigint;
    rounds: number | bigint;
  };
}
