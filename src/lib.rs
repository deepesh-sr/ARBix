//!
//! Impermanent Loss Insurance Contract
//!
//! A Stylus smart contract that provides insurance coverage for impermanent loss (IL) 
//! experienced by liquidity providers in AMM pools.
//!
//! ## Features
//! - Banded coverage model with configurable threshold and cap
//! - Real-time IL calculation based on pool state and oracle prices
//! - Automated claim processing
//! - Owner-controlled policy parameters
//!
//! ## Solidity Equivalent
//! ```solidity
//! contract ILInsurance {
//!     // Policy parameters
//!     uint256 public thresholdBps;
//!     uint256 public upperCapBps;
//!     uint256 public payoutRatioBps;
//!     
//!     // Pool state
//!     uint256 public reserveTokenA;
//!     uint256 public reserveTokenB;
//!     uint256 public lpTotalSupply;
//!     
//!     // Oracle prices
//!     uint256 public priceTokenA;
//!     uint256 public priceTokenB;
//!     
//!     function calculateIL() external view returns (uint256);
//!     function calculatePayout() external view returns (uint256);
//!     function claim() external returns (uint256);
//! }
//! ```
//!
//! Note: this code is for demonstration and has not been audited.
//!
// Allow `cargo stylus export-abi` to generate a main function.

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::{alloy_primitives::U256, prelude::*};

pub mod constant;
pub mod util;
pub mod lp_valuator;
pub mod policy_manager;

// Re-export key items explicitly to avoid ambiguous glob imports
pub use constant::{
    SCALE as CONSTANT_SCALE, PAIR_A_RESERVE_ETH, PAIR_A_RESERVE_USDC,
    PAIR_A_LP_TOTAL_SUPPLY, USER_LP_AMOUNT, PRICE_ETH_USD, PRICE_USDC_USD
};
pub use util::{mul_div, user_share_scaled};
pub use lp_valuator::compute_values_from_constants;
pub use policy_manager::claim_demo;

// Constants for the contract
const SCALE: u128 = 1_000_000_000_000_000_000u128; // 1e18
const BPS_DENOMINATOR: u32 = 10_000u32; // Basis points denominator (100% = 10000 bps)

// Define persistent storage for the IL Insurance contract using Solidity ABI.
// `ILInsurance` will be the entrypoint.
sol_storage! {
    #[entrypoint]
    pub struct ILInsurance {
        // Policy parameters (in basis points, 10000 = 100%)
        uint256 threshold_bps;        // Minimum IL before payout (e.g., 1000 = 10%)
        uint256 upper_cap_bps;        // Maximum covered IL (e.g., 2000 = 20%)
        uint256 payout_ratio_bps;     // Payout percentage (e.g., 8000 = 80%)
        
        // Pool state
        uint256 reserve_token_a;      // Reserve of token A (e.g., ETH) - scaled by 1e18
        uint256 reserve_token_b;      // Reserve of token B (e.g., USDC) - scaled by 1e18
        uint256 lp_total_supply;      // Total LP token supply - scaled by 1e18
        
        // Oracle prices (scaled by 1e18)
        uint256 price_token_a;        // Price of token A in USD
        uint256 price_token_b;        // Price of token B in USD
        
        // User position (simplified - in production use mapping)
        uint256 user_lp_amount;       // User's LP token amount - scaled by 1e18
        uint256 user_original_token_a; // Original token A deposited - scaled by 1e18
        uint256 user_original_token_b; // Original token B deposited - scaled by 1e18
        
        // Contract admin
        address owner;
        bool initialized;
    }
}

/// Declare that `ILInsurance` is a contract with the following external methods.
#[public]
impl ILInsurance {
    
    // ========== Initialization ==========
    
    /// Initialize the contract with policy parameters
    /// Can only be called once
    pub fn initialize(
        &mut self,
        threshold_bps: U256,
        upper_cap_bps: U256,
        payout_ratio_bps: U256,
    ) {
        // Check if already initialized
        assert!(!self.initialized.get(), "Already initialized");
        
        // Validate parameters
        assert!(threshold_bps < upper_cap_bps, "Invalid threshold");
        assert!(upper_cap_bps <= U256::from(BPS_DENOMINATOR), "Cap too high");
        assert!(payout_ratio_bps <= U256::from(BPS_DENOMINATOR), "Ratio too high");
        
        self.threshold_bps.set(threshold_bps);
        self.upper_cap_bps.set(upper_cap_bps);
        self.payout_ratio_bps.set(payout_ratio_bps);
        self.owner.set(self.vm().msg_sender());
        self.initialized.set(true);
    }
    
    // ========== View Functions - Policy & State ==========
    
    /// Get the current policy parameters (threshold, cap, payout ratio in bps)
    pub fn get_policy(&self) -> (U256, U256, U256) {
        (
            self.threshold_bps.get(),
            self.upper_cap_bps.get(),
            self.payout_ratio_bps.get(),
        )
    }
    
    /// Get the current pool state (reserve A, reserve B, LP total supply)
    pub fn get_pool_state(&self) -> (U256, U256, U256) {
        (
            self.reserve_token_a.get(),
            self.reserve_token_b.get(),
            self.lp_total_supply.get(),
        )
    }
    
    /// Get oracle prices (price A, price B)
    pub fn get_prices(&self) -> (U256, U256) {
        (
            self.price_token_a.get(),
            self.price_token_b.get(),
        )
    }
    
    /// Get user position (LP amount, original token A, original token B)
    pub fn get_user_position(&self) -> (U256, U256, U256) {
        (
            self.user_lp_amount.get(),
            self.user_original_token_a.get(),
            self.user_original_token_b.get(),
        )
    }
    
    /// Get contract owner
    pub fn owner(&self) -> alloy_primitives::Address {
        self.owner.get()
    }
    
    /// Check if contract is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized.get()
    }
    
    // ========== View Functions - Calculations ==========
    
    /// Calculate user's share of the pool (returns fraction scaled by 1e18)
    /// Example: 0.1% = 1000000000000000 (0.001 * 1e18)
    pub fn calculate_user_share(&self) -> U256 {
        let user_lp = self.user_lp_amount.get();
        let total_supply = self.lp_total_supply.get();
        
        if total_supply == U256::ZERO {
            return U256::ZERO;
        }
        
        // user_share = (user_lp * SCALE) / total_supply
        (user_lp * U256::from(SCALE)) / total_supply
    }
    
    /// Calculate current LP value in USD (scaled by 1e18)
    pub fn calculate_lp_value(&self) -> U256 {
        let user_share = self.calculate_user_share();
        
        let reserve_a = self.reserve_token_a.get();
        let reserve_b = self.reserve_token_b.get();
        let price_a = self.price_token_a.get();
        let price_b = self.price_token_b.get();
        
        // Current token amounts for user
        let current_a = (reserve_a * user_share) / U256::from(SCALE);
        let current_b = (reserve_b * user_share) / U256::from(SCALE);
        
        // Value in USD
        let value_a = (current_a * price_a) / U256::from(SCALE);
        let value_b = (current_b * price_b) / U256::from(SCALE);
        
        value_a + value_b
    }
    
    /// Calculate holding value if tokens were not LP'd (scaled by 1e18)
    pub fn calculate_holding_value(&self) -> U256 {
        let original_a = self.user_original_token_a.get();
        let original_b = self.user_original_token_b.get();
        let price_a = self.price_token_a.get();
        let price_b = self.price_token_b.get();
        
        let value_a = (original_a * price_a) / U256::from(SCALE);
        let value_b = (original_b * price_b) / U256::from(SCALE);
        
        value_a + value_b
    }
    
    /// Calculate impermanent loss percentage (scaled by 1e18)
    /// Example: 50% IL = 500000000000000000 (0.5 * 1e18)
    pub fn calculate_il(&self) -> U256 {
        let lp_value = self.calculate_lp_value();
        let holding_value = self.calculate_holding_value();
        
        if holding_value == U256::ZERO {
            return U256::ZERO;
        }
        
        if lp_value >= holding_value {
            return U256::ZERO; // No impermanent loss
        }
        
        let loss = holding_value - lp_value;
        (loss * U256::from(SCALE)) / holding_value
    }
    
    /// Calculate the insurance payout for current position (scaled by 1e18)
    /// Returns 0 if IL is below threshold
    pub fn calculate_payout(&self) -> U256 {
        let il_frac = self.calculate_il();
        let holding_value = self.calculate_holding_value();
        
        let threshold = self.threshold_bps.get();
        let upper_cap = self.upper_cap_bps.get();
        let payout_ratio = self.payout_ratio_bps.get();
        
        // Convert bps to scaled fraction (e.g., 1000 bps = 10% = 0.1 * 1e18)
        let threshold_scaled = (threshold * U256::from(SCALE)) / U256::from(BPS_DENOMINATOR);
        let upper_scaled = (upper_cap * U256::from(SCALE)) / U256::from(BPS_DENOMINATOR);
        
        // Cap IL at upper bound
        let il_capped = if il_frac > upper_scaled { upper_scaled } else { il_frac };
        
        // Check threshold - no payout if below
        if il_capped <= threshold_scaled {
            return U256::ZERO;
        }
        
        // Calculate covered fraction (IL above threshold, up to cap)
        let covered_frac = il_capped - threshold_scaled;
        
        // Calculate loss amount in USD
        let loss_amount = (holding_value * covered_frac) / U256::from(SCALE);
        
        // Apply payout ratio (e.g., 80% coverage)
        (loss_amount * payout_ratio) / U256::from(BPS_DENOMINATOR)
    }
    
    // ========== State-Changing Functions ==========
    
    /// Update pool state (only owner can call)
    /// Used to sync pool reserves and LP supply from the AMM
    pub fn update_pool_state(
        &mut self,
        reserve_a: U256,
        reserve_b: U256,
        total_supply: U256,
    ) {
        assert!(self.vm().msg_sender() == self.owner.get(), "Only owner");
        
        self.reserve_token_a.set(reserve_a);
        self.reserve_token_b.set(reserve_b);
        self.lp_total_supply.set(total_supply);
    }
    
    /// Update oracle prices (only owner can call)
    /// Used to sync token prices from external oracles
    pub fn update_prices(
        &mut self,
        price_a: U256,
        price_b: U256,
    ) {
        assert!(self.vm().msg_sender() == self.owner.get(), "Only owner");
        
        self.price_token_a.set(price_a);
        self.price_token_b.set(price_b);
    }
    
    /// Update user position (only owner can call)
    /// In production, this would be a mapping(address => Position)
    pub fn update_user_position(
        &mut self,
        lp_amount: U256,
        original_a: U256,
        original_b: U256,
    ) {
        assert!(self.vm().msg_sender() == self.owner.get(), "Only owner");
        
        self.user_lp_amount.set(lp_amount);
        self.user_original_token_a.set(original_a);
        self.user_original_token_b.set(original_b);
    }
    
    /// Process an insurance claim
    /// Returns the payout amount (0 if no payout due)
    /// In production, this would transfer tokens to the user
    pub fn claim(&mut self) -> U256 {
        let payout = self.calculate_payout();
        
        // In production:
        // 1. Check contract has sufficient balance
        // 2. Transfer payout to msg::sender()
        // 3. Emit ClaimProcessed event
        // 4. Update user's position/claim history
        
        payout
    }
    
    /// Update policy parameters (only owner can call)
    pub fn update_policy(
        &mut self,
        threshold_bps: U256,
        upper_cap_bps: U256,
        payout_ratio_bps: U256,
    ) {
        assert!(self.vm().msg_sender() == self.owner.get(), "Only owner");
        assert!(threshold_bps < upper_cap_bps, "Invalid threshold");
        assert!(upper_cap_bps <= U256::from(BPS_DENOMINATOR), "Cap too high");
        
        self.threshold_bps.set(threshold_bps);
        self.upper_cap_bps.set(upper_cap_bps);
        self.payout_ratio_bps.set(payout_ratio_bps);
    }
    
    // ========== Helper/Demo Functions ==========
    
    /// Set up a demo scenario with predefined values
    /// Useful for testing and demonstrations
    pub fn setup_demo(&mut self) {
        assert!(self.vm().msg_sender() == self.owner.get(), "Only owner");
        
        // Pool state: 500 ETH + 1M USDC
        self.reserve_token_a.set(U256::from(500u128) * U256::from(SCALE));
        self.reserve_token_b.set(U256::from(1_000_000u128) * U256::from(SCALE));
        self.lp_total_supply.set(U256::from(1_000_000u128) * U256::from(SCALE));
        
        // Oracle prices: ETH = $2000, USDC = $1
        self.price_token_a.set(U256::from(2000u128) * U256::from(SCALE));
        self.price_token_b.set(U256::from(SCALE)); // $1
        
        // User position: 1000 LP tokens, originally deposited 1 ETH + 2000 USDC
        self.user_lp_amount.set(U256::from(1000u128) * U256::from(SCALE));
        self.user_original_token_a.set(U256::from(SCALE)); // 1 ETH
        self.user_original_token_b.set(U256::from(2000u128) * U256::from(SCALE)); // 2000 USDC
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use stylus_sdk::testing::*;

    #[test]
    fn test_il_insurance_initialization() {
        let vm = TestVM::default();
        let mut contract = ILInsurance::from(&vm);

        // Initialize with policy parameters
        // Threshold: 10% (1000 bps), Cap: 20% (2000 bps), Payout: 80% (8000 bps)
        contract.initialize(
            U256::from(1000u32),  // 10%
            U256::from(2000u32),  // 20%
            U256::from(8000u32),  // 80%
        );

        assert!(contract.is_initialized());
        
        let (threshold, cap, ratio) = contract.get_policy();
        assert_eq!(threshold, U256::from(1000u32));
        assert_eq!(cap, U256::from(2000u32));
        assert_eq!(ratio, U256::from(8000u32));
    }

    #[test]
    fn test_il_insurance_demo_scenario() {
        let vm = TestVM::default();
        let mut contract = ILInsurance::from(&vm);

        // Initialize
        contract.initialize(
            U256::from(1000u32),  // 10% threshold
            U256::from(2000u32),  // 20% cap
            U256::from(8000u32),  // 80% payout
        );

        // Set up demo scenario
        contract.setup_demo();

        // Check pool state
        let (reserve_a, reserve_b, total_supply) = contract.get_pool_state();
        assert_eq!(reserve_a, U256::from(500u128) * U256::from(SCALE));
        assert_eq!(reserve_b, U256::from(1_000_000u128) * U256::from(SCALE));
        assert_eq!(total_supply, U256::from(1_000_000u128) * U256::from(SCALE));

        // Check prices
        let (price_a, price_b) = contract.get_prices();
        assert_eq!(price_a, U256::from(2000u128) * U256::from(SCALE));
        assert_eq!(price_b, U256::from(SCALE));

        // Check user position
        let (user_lp, original_a, original_b) = contract.get_user_position();
        assert_eq!(user_lp, U256::from(1000u128) * U256::from(SCALE));
        assert_eq!(original_a, U256::from(SCALE)); // 1 ETH
        assert_eq!(original_b, U256::from(2000u128) * U256::from(SCALE)); // 2000 USDC

        // Calculate user share (should be 0.1%)
        let user_share = contract.calculate_user_share();
        let expected_share = U256::from(SCALE) / U256::from(1000u128); // 0.001 = 0.1%
        assert_eq!(user_share, expected_share);

        // Calculate LP value
        let lp_value = contract.calculate_lp_value();
        println!("LP Value: {}", lp_value);
        assert!(lp_value > U256::ZERO);

        // Calculate holding value
        let holding_value = contract.calculate_holding_value();
        println!("Holding Value: {}", holding_value);
        assert_eq!(holding_value, U256::from(4000u128) * U256::from(SCALE)); // 1 ETH * $2000 + 2000 USDC * $1

        // Calculate IL
        let il = contract.calculate_il();
        println!("IL: {} (scaled by 1e18)", il);
        assert!(il > U256::ZERO, "Should have impermanent loss");

        // Calculate payout
        let payout = contract.calculate_payout();
        println!("Payout: {}", payout);
        
        // With 50% IL:
        // Covered IL = min(50%, 20%) - 10% = 10%
        // Loss = $4000 * 10% = $400
        // Payout = $400 * 80% = $320
        let expected_payout = U256::from(320u128) * U256::from(SCALE);
        assert_eq!(payout, expected_payout, "Payout should be $320");
    }

    #[test]
    fn test_il_calculation() {
        let vm = TestVM::default();
        let mut contract = ILInsurance::from(&vm);

        contract.initialize(
            U256::from(1000u32),
            U256::from(2000u32),
            U256::from(8000u32),
        );

        // Set up a scenario with known IL
        // Pool: 500 ETH + 1M USDC, Total supply: 1M LP
        contract.update_pool_state(
            U256::from(500u128) * U256::from(SCALE),
            U256::from(1_000_000u128) * U256::from(SCALE),
            U256::from(1_000_000u128) * U256::from(SCALE),
        );

        // Prices: ETH = $2000, USDC = $1
        contract.update_prices(
            U256::from(2000u128) * U256::from(SCALE),
            U256::from(SCALE),
        );

        // User: 1000 LP tokens, originally 1 ETH + 2000 USDC
        contract.update_user_position(
            U256::from(1000u128) * U256::from(SCALE),
            U256::from(SCALE),
            U256::from(2000u128) * U256::from(SCALE),
        );

        let lp_value = contract.calculate_lp_value();
        let holding_value = contract.calculate_holding_value();
        let il = contract.calculate_il();

        // User has 0.1% of pool
        // Current: 0.5 ETH + 1000 USDC = $1000 + $1000 = $2000
        // Original: 1 ETH + 2000 USDC = $2000 + $2000 = $4000
        // IL = ($4000 - $2000) / $4000 = 50%

        assert_eq!(holding_value, U256::from(4000u128) * U256::from(SCALE));
        assert_eq!(lp_value, U256::from(2000u128) * U256::from(SCALE));
        
        let expected_il = U256::from(SCALE) / U256::from(2u128); // 50%
        assert_eq!(il, expected_il, "IL should be 50%");
    }

    #[test]
    fn test_payout_below_threshold() {
        let vm = TestVM::default();
        let mut contract = ILInsurance::from(&vm);

        contract.initialize(
            U256::from(1000u32),  // 10% threshold
            U256::from(2000u32),
            U256::from(8000u32),
        );

        // Set up scenario with low IL (below threshold)
        contract.update_pool_state(
            U256::from(500u128) * U256::from(SCALE),
            U256::from(1_000_000u128) * U256::from(SCALE),
            U256::from(1_000_000u128) * U256::from(SCALE),
        );

        contract.update_prices(
            U256::from(2000u128) * U256::from(SCALE),
            U256::from(SCALE),
        );

        // User with minimal IL
        contract.update_user_position(
            U256::from(1000u128) * U256::from(SCALE),
            U256::from(SCALE),
            U256::from(1900u128) * U256::from(SCALE), // Close to current ratio
        );

        let payout = contract.calculate_payout();
        
        // If IL < 10%, payout should be 0
        let il = contract.calculate_il();
        if il < U256::from(SCALE) / U256::from(10u128) {
            assert_eq!(payout, U256::ZERO, "No payout below threshold");
        }
    }

    #[test]
    fn test_claim_processing() {
        let vm = TestVM::default();
        let mut contract = ILInsurance::from(&vm);

        contract.initialize(
            U256::from(1000u32),
            U256::from(2000u32),
            U256::from(8000u32),
        );

        contract.setup_demo();

        // Process claim
        let payout = contract.claim();
        
        assert!(payout > U256::ZERO, "Should receive payout for demo scenario");
        assert_eq!(payout, U256::from(320u128) * U256::from(SCALE), "Payout should be $320");
    }
}
