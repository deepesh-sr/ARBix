// utils.rs
use stylus_sdk::alloy_primitives::U256;

pub const SCALE: u128 = crate::constant::SCALE;

// Using U256 from Stylus SDK for safe arithmetic without overflow
pub fn mul_div(a: u128, b: u128, denom: u128) -> u128 {
    // Safely compute (a * b) / denom using U256 to avoid overflow
    
    if denom == 0 {
        return 0; // avoid division by zero
    }
    
    // Convert to U256 for safe arithmetic
    let a_u256 = U256::from(a);
    let b_u256 = U256::from(b);
    let denom_u256 = U256::from(denom);
    
    // Perform calculation with U256 (no overflow possible)
    let result = (a_u256 * b_u256) / denom_u256;
    
    // Convert back to u128, saturating if result is too large
    // This shouldn't happen in our use case but provides safety
    if result > U256::from(u128::MAX) {
        u128::MAX
    } else {
        result.to::<u128>()
    }
}

// compute user share: lp_amount / total_supply, scaled by SCALE
pub fn user_share_scaled(lp_amount: u128, total_supply: u128) -> u128 {
    mul_div(lp_amount, SCALE, total_supply)
}
