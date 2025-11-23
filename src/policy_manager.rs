// policy_manager.rs (claim using constants)
use crate::lp_valuator::compute_values_from_constants;
use crate::mul_div;
use crate::util::SCALE;

pub fn claim_demo() -> (u128 /*payout_usd*/, u128 /*il_frac*/) {
    let (_lp_value, holding_value, il_frac) = compute_values_from_constants();

    // banded coverage example: T=10% (1000 bps), U=20% (2000 bps), R=80%
    let threshold_bps = 1000u128;
    let upper_bps = 2000u128;
    let payout_ratio_bps = 8000u128;

    // convert bps to scaled fraction: bps/10000 scaled by SCALE
    let threshold_scaled = threshold_bps * (SCALE / 10000u128);
    let upper_scaled = upper_bps * (SCALE / 10000u128);

    let il_capped = if il_frac > upper_scaled { upper_scaled } else { il_frac };
    if il_capped <= threshold_scaled {
        return (0u128, il_frac); // no payout
    }
    let covered_frac = il_capped - threshold_scaled; // scaled
    let loss_amount = mul_div(holding_value, covered_frac, SCALE); // in USD scaled
    let payout = mul_div(loss_amount, payout_ratio_bps, 10000u128); // in USD scaled

    (payout, il_frac)
}
