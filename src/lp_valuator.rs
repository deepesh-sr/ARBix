// lp_valuator.rs
use crate::constant::*;
use crate::util::SCALE;
use crate::util::*;


pub fn compute_values_from_constants() -> (u128 /*lp_value*/, u128 /*holding_value*/, u128 /*il_frac*/) {
    // read constants
    let reserve_eth = PAIR_A_RESERVE_ETH;
    let reserve_usdc = PAIR_A_RESERVE_USDC;
    let total_supply = PAIR_A_LP_TOTAL_SUPPLY;
    let user_lp = USER_LP_AMOUNT;

    // user share: user_lp / total_supply (both already scaled, so result is fraction scaled by SCALE)
    let user_share = mul_div(user_lp, SCALE, total_supply); // scaled by SCALE

    // current underlying token amounts for the user
    let current_eth = mul_div(reserve_eth, user_share, SCALE);
    let current_usdc = mul_div(reserve_usdc, user_share, SCALE);

    // current LP value in USD (scaled)
    let lp_value_usd = mul_div(current_eth, PRICE_ETH_USD, SCALE)
                     + mul_div(current_usdc, PRICE_USDC_USD, SCALE);

    // holding value: use snapshot constants (for demo we can reuse same original amounts or provide different)
    // For simplicity: assume original_a = user_share_at_buy * reserve_at_buy (we can hardcode buy snapshot)
    // Example: snapshot reserves at buy time (hardcoded different)
    let original_eth = 1 * SCALE; // user originally had 1 ETH (sample)
    let original_usdc = 2000 * SCALE;
    let holding_value_usd = mul_div(original_eth, PRICE_ETH_USD, SCALE)
                          + mul_div(original_usdc, PRICE_USDC_USD, SCALE);

    // compute IL
    let diff = if holding_value_usd > lp_value_usd { holding_value_usd - lp_value_usd } else { 0 };
    let il_frac = if holding_value_usd == 0 { 0 } else { mul_div(diff, SCALE, holding_value_usd) };

    (lp_value_usd, holding_value_usd, il_frac)
}
