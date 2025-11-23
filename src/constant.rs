// constants.rs
pub const SCALE: u128 = 1_000_000_000_000_000_000u128; // 1e18

// Dummy pool state for pair A (ETH-USDC) — values in token smallest units
// Reserve units: token amounts scaled by 1e18 (so 2 ETH -> 2 * 1e18)
pub const PAIR_A_RESERVE_ETH: u128 = 500 * SCALE;      // 500 ETH
pub const PAIR_A_RESERVE_USDC: u128 = 1_000_000 * SCALE; // 1,000,000 USDC

// LP token total supply (scaled like token units)
pub const PAIR_A_LP_TOTAL_SUPPLY: u128 = 1_000_000 * SCALE; // e.g., 1,000,000 LP units

// Example user LP amount locked (e.g., 1000 LP)
pub const USER_LP_AMOUNT: u128 = 1000 * SCALE; // user holds 1000 LP

// Oracle prices scaled by 1e18: price in USD per token unit
pub const PRICE_ETH_USD: u128 = 2000 * SCALE; // $2000 per ETH
pub const PRICE_USDC_USD: u128 = 1 * SCALE;   // $1 per USDC
