pub cp_swap_math(
    amount_in: u64,
    fee_bps: u64
) -> u64 {
    let reserve_x = 1000000;
    let reserve_y = 2000000;
    require(reserve_x > 0);
    require(reserve_y > 0);
    require(amount_in > 0);
    require(fee_bps <= 1000);

    let fee = amount_in * fee_bps / 10000;
    let amount_in_net = amount_in - fee;
    let k = reserve_x * reserve_y;
    let new_x = reserve_x + amount_in_net;
    require(new_x > 0);

    let new_y = k / new_x;
    require(reserve_y > new_y);

    return reserve_y - new_y;
}

pub accrue_interest_math(
    debt: u64,
    rate_bps: u64,
    slots: u64
) -> u64 {
    let collateral = 3000000;
    require(rate_bps <= 5000);
    require(slots > 0);

    let util = debt * 10000 / (collateral + 1);
    let base_interest = debt * rate_bps / 10000;
    let total_interest = base_interest * slots;
    return debt + total_interest + util;
}

pub recursive_sum(depth: u64, acc: u64) -> u64 {
    if (depth == 0) {
        return acc;
    }
    return recursive_sum(depth - 1, acc + depth);
}

pub recursive_bounded(depth: u64) -> u64 {
    require(depth <= 7);
    return recursive_sum(depth, 0);
}

pub recursive_error(depth: u64) -> u64 {
    require(depth <= 4);
    return recursive_sum(depth, 0);
}

pub stable_swap_invariant_iterative(
    reserve_x: u64,
    reserve_y: u64,
    amp_factor: u64,
    iterations: u64
) -> u64 {
    require(reserve_x > 0);
    require(reserve_y > 0);
    require(amp_factor >= 1);
    require(amp_factor <= 10000);
    require(iterations > 0);
    require(iterations <= 8);

    let mut d = reserve_x + reserve_y;
    let mut i = 0;

    while (i < iterations) {
        let prod = reserve_x * reserve_y;
        require(prod > 0);
        require(d > 0);

        let ann = amp_factor * 2;
        let term1 = ann * (reserve_x + reserve_y);
        let term2 = prod * 4 / d;
        let numerator = (term1 + term2) * d;

        let term3 = (ann - 1) * d;
        let term4 = prod * 8 / d;
        let denominator = term3 + term4;
        require(denominator > 0);

        d = numerator / denominator;
        i = i + 1;
    }

    return d;
}

pub utilization_kink_rate(
    util_bps: u64,
    base_rate_bps: u64,
    slope_low_bps: u64,
    slope_high_bps: u64,
    kink_bps: u64
) -> u64 {
    require(util_bps <= 10000);
    require(kink_bps > 0);
    require(kink_bps <= 10000);

    let mut rate = base_rate_bps;
    if (util_bps <= kink_bps) {
        rate = rate + (util_bps * slope_low_bps / kink_bps);
    } else {
        let normal = slope_low_bps;
        let excess_util = util_bps - kink_bps;
        let excess_den = 10000 - kink_bps;
        let excess = excess_util * slope_high_bps / excess_den;
        rate = rate + normal + excess;
    }

    return rate;
}

pub funding_rate_path(
    skew_bps: u64,
    base_rate_bps: u64,
    velocity_bps: u64,
    intervals: u64
) -> u64 {
    require(skew_bps <= 10000);
    require(intervals > 0);
    require(intervals <= 12);

    let mut rate = base_rate_bps;
    let mut i = 0;

    while (i < intervals) {
        if (skew_bps > 5000) {
            rate = rate + velocity_bps;
        } else {
            if (rate > velocity_bps) {
                rate = rate - velocity_bps;
            }
        }

        // Mild damping per interval to mimic mean reversion.
        rate = rate * 9990 / 10000;
        i = i + 1;
    }

    return rate;
}

pub collateral_health_loop(
    collateral: u64,
    debt: u64,
    liquidation_threshold_bps: u64,
    haircut_bps: u64,
    rounds: u64
) -> u64 {
    require(collateral > 0);
    require(debt > 0);
    require(liquidation_threshold_bps <= 10000);
    require(haircut_bps <= 10000);
    require(rounds > 0);
    require(rounds <= 8);

    let mut adjusted_collateral = collateral;
    let mut i = 0;

    while (i < rounds) {
        adjusted_collateral = adjusted_collateral * haircut_bps / 10000;
        i = i + 1;
    }

    let borrow_limit = adjusted_collateral * liquidation_threshold_bps / 10000;
    require(borrow_limit > 0);
    return borrow_limit * 10000 / debt;
}
