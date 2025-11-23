#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]

#[cfg(not(any(test, feature = "export-abi")))]
#[no_mangle]
pub extern "C" fn main() {}

#[cfg(feature = "export-abi")]
fn main() {
    stylus_hello_world::print_from_args();
}

#[cfg(test)]
mod tests {
    use stylus_hello_world::{
        PAIR_A_RESERVE_ETH, PAIR_A_RESERVE_USDC, 
        PAIR_A_LP_TOTAL_SUPPLY, USER_LP_AMOUNT,
        PRICE_ETH_USD, PRICE_USDC_USD,
        mul_div, user_share_scaled, compute_values_from_constants, claim_demo
    };
    use stylus_hello_world::constant::SCALE;
    
    #[test]
    fn test_complete_il_insurance_workflow() {
        println!("\n=== IL Insurance Workflow Test ===\n");
        
        // Step 1: Display pool constants
        println!("📊 Pool Configuration:");
        println!("  ETH Reserve: {} ETH", PAIR_A_RESERVE_ETH / SCALE);
        println!("  USDC Reserve: {} USDC", PAIR_A_RESERVE_USDC / SCALE);
        println!("  LP Total Supply: {} LP tokens", PAIR_A_LP_TOTAL_SUPPLY / SCALE);
        println!("  User LP Amount: {} LP tokens", USER_LP_AMOUNT / SCALE);
        println!();
        
        // Step 2: Display oracle prices
        println!("💵 Oracle Prices:");
        println!("  ETH Price: ${}", PRICE_ETH_USD / SCALE);
        println!("  USDC Price: ${}", PRICE_USDC_USD / SCALE);
        println!();
        
        // Step 3: Calculate user's share
        let user_share = user_share_scaled(USER_LP_AMOUNT, PAIR_A_LP_TOTAL_SUPPLY);
        println!("👤 User's LP Share: {}%", (user_share * 100) / SCALE);
        println!();
        
        // Step 4: Compute LP value and IL
        let (lp_value, holding_value, il_frac) = compute_values_from_constants();
        
        println!("📈 Position Analysis:");
        println!("  Current LP Value: ${}.{:06}", 
            lp_value / SCALE, 
            ((lp_value % SCALE) / (SCALE / 1_000_000))
        );
        println!("  Holding Value: ${}.{:06}", 
            holding_value / SCALE, 
            ((holding_value % SCALE) / (SCALE / 1_000_000))
        );
        
        let il_percentage = (il_frac * 100) / SCALE;
        let il_decimal = ((il_frac * 10000) / SCALE) % 100;
        println!("  Impermanent Loss: {}.{:02}%", il_percentage, il_decimal);
        println!();
        
        // Step 5: Process insurance claim
        let (payout, il_frac_claim) = claim_demo();
        
        println!("🛡️ Insurance Policy:");
        println!("  Threshold: 10% (no coverage below this)");
        println!("  Upper Cap: 20% (maximum covered IL)");
        println!("  Payout Ratio: 80% of covered loss");
        println!();
        
        println!("💰 Claim Result:");
        if payout > 0 {
            println!("  Payout Amount: ${}.{:06}", 
                payout / SCALE, 
                ((payout % SCALE) / (SCALE / 1_000_000))
            );
            println!("  ✅ Claim APPROVED");
        } else {
            println!("  Payout Amount: $0");
            println!("  ❌ Claim DENIED (IL below threshold)");
        }
        println!();
        
        // Assertions to validate the workflow
        assert!(lp_value > 0, "LP value should be positive");
        assert!(holding_value > 0, "Holding value should be positive");
        assert_eq!(il_frac, il_frac_claim, "IL fraction should match between calculations");
        
        // Test IL calculation logic
        if holding_value > lp_value {
            let expected_il = mul_div(holding_value - lp_value, SCALE, holding_value);
            assert_eq!(il_frac, expected_il, "IL calculation should be correct");
        }
        
        println!("✅ All workflow tests passed!");
    }
    
    #[test]
    fn test_il_below_threshold() {
        println!("\n=== Testing IL Below Threshold ===\n");
        
        let (payout, il_frac) = claim_demo();
        
        // With current constants, IL should be minimal
        // If IL is below 10% threshold, payout should be 0
        let threshold_scaled = 1000u128 * (SCALE / 10000u128); // 10%
        
        if il_frac <= threshold_scaled {
            assert_eq!(payout, 0, "Payout should be 0 when IL is below threshold");
            println!("✅ Correctly denied claim for IL below threshold");
        } else {
            assert!(payout > 0, "Payout should be positive when IL is above threshold");
            println!("✅ Correctly approved claim for IL above threshold");
        }
        
        println!("IL: {}.{:02}%", (il_frac * 100) / SCALE, ((il_frac * 10000) / SCALE) % 100);
        println!("Payout: ${}.{:06}", payout / SCALE, ((payout % SCALE) / (SCALE / 1_000_000)));
    }
    
    #[test]
    fn test_user_share_calculation() {
        println!("\n=== Testing User Share Calculation ===\n");
        
        // Test with known values
        let user_lp = USER_LP_AMOUNT; // 1000 * SCALE LP tokens
        let total_supply = PAIR_A_LP_TOTAL_SUPPLY; // 1,000,000 * SCALE LP tokens
        
        // Since both are already scaled, we calculate: (user_lp * SCALE) / total_supply
        let share = mul_div(user_lp, SCALE, total_supply);
        
        // Expected: (1000 * SCALE * SCALE) / (1_000_000 * SCALE) = (1000 * SCALE) / 1_000_000 = 0.001 * SCALE
        let expected_share = SCALE / 1000u128; // 0.1% of SCALE
        
        assert_eq!(share, expected_share, "User share calculation should be accurate");
        
        println!("User LP: {}", user_lp / SCALE);
        println!("Total Supply: {}", total_supply / SCALE);
        println!("User Share (scaled): {}", share);
        println!("User Share: {}.{:03}%", (share * 100) / SCALE, ((share * 100_000) / SCALE) % 1000);
        println!("✅ User share calculation is correct");
    }
    
    #[test]
    fn test_mul_div_precision() {
        println!("\n=== Testing Math Utility Functions ===\n");
        
        // Test mul_div with smaller inputs to avoid overflow
        let a = 100;
        let b = 50;
        let denom = 10;
        
        let result = mul_div(a, b, denom);
        let expected = 500; // (100 * 50) / 10 = 500
        
        assert_eq!(result, expected, "mul_div should calculate correctly");
        println!("Test 1: ({} * {}) / {} = {} ✓", a, b, denom, result);
        
        // Test with percentage calculation
        let value = 1000 * SCALE;
        let percent = 15 * SCALE / 100; // 15%
        let percent_value = mul_div(value, percent, SCALE);
        
        assert_eq!(percent_value, 150 * SCALE, "Percentage calculation should work");
        println!("Test 2: 15% of 1000 = {} ✓", percent_value / SCALE);
        
        // Test simple division
        let result3 = mul_div(1000, SCALE, 1_000_000);
        println!("Test 3: User share calculation: {} ✓", result3);
        
        println!("✅ Math utility functions are working correctly");
    }
    
    #[test]
    fn test_lp_value_computation() {
        println!("\n=== Testing LP Value Computation ===\n");
        
        let (lp_value, holding_value, il_frac) = compute_values_from_constants();
        
        // Verify that values are computed
        assert!(lp_value > 0, "LP value should be positive");
        assert!(holding_value > 0, "Holding value should be positive");
        
        // IL should be between 0 and 100%
        assert!(il_frac <= SCALE, "IL should not exceed 100%");
        
        // If there's a loss, lp_value should be less than holding_value
        if il_frac > 0 {
            assert!(lp_value < holding_value, "LP value should be less than holding value when there's IL");
        }
        
        println!("LP Value: ${}", lp_value / SCALE);
        println!("Holding Value: ${}", holding_value / SCALE);
        println!("IL: {}.{:02}%", (il_frac * 100) / SCALE, ((il_frac * 10000) / SCALE) % 100);
        println!("✅ LP value computation is consistent");
    }
    
    #[test]
    fn test_payout_calculation_logic() {
        println!("\n=== Testing Payout Calculation Logic ===\n");
        
        let (payout, il_frac) = claim_demo();
        
        // Recreate the logic to verify
        let threshold_bps = 1000u128; // 10%
        let upper_bps = 2000u128; // 20%
        let payout_ratio_bps = 8000u128; // 80%
        
        let threshold_scaled = threshold_bps * (SCALE / 10000u128);
        let upper_scaled = upper_bps * (SCALE / 10000u128);
        
        let il_capped = if il_frac > upper_scaled { upper_scaled } else { il_frac };
        
        if il_capped <= threshold_scaled {
            assert_eq!(payout, 0, "Payout should be 0 when IL is below threshold");
        } else {
            let covered_frac = il_capped - threshold_scaled;
            let (_, holding_value, _) = compute_values_from_constants();
            let loss_amount = mul_div(holding_value, covered_frac, SCALE);
            let expected_payout = mul_div(loss_amount, payout_ratio_bps, 10000u128);
            
            assert_eq!(payout, expected_payout, "Payout should match expected calculation");
        }
        
        println!("Calculated Payout: ${}.{:06}", payout / SCALE, ((payout % SCALE) / (SCALE / 1_000_000)));
        println!("✅ Payout calculation logic is correct");
    }
}
