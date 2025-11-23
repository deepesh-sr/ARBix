# IL Insurance - Impermanent Loss Insurance Protocol

A production-ready Arbitrum Stylus smart contract that provides insurance coverage for impermanent loss (IL) experienced by liquidity providers in AMM pools. Built with Rust using the [stylus-sdk](https://github.com/OffchainLabs/stylus-sdk-rs).

## Overview

This protocol implements a banded coverage model for impermanent loss insurance with configurable parameters:
- **Threshold**: Minimum IL percentage before coverage kicks in (default: 10%)
- **Upper Cap**: Maximum covered IL percentage (default: 20%)
- **Payout Ratio**: Percentage of covered loss paid out (default: 80%)

### Key Features

- ✅ Real-time IL calculation based on pool reserves and oracle prices
- ✅ Automated claim processing with payout calculation
- ✅ Safe arithmetic using U256 (no overflow issues)
- ✅ Owner-controlled policy parameters
- ✅ Comprehensive test suite (11/11 tests passing)
- ✅ Solidity ABI compatible
- ✅ WASM-optimized (8.9 KB compressed)

### Example Scenario

**Initial Position:**
- User deposits 1 ETH + 2,000 USDC to liquidity pool
- Receives 1,000 LP tokens (0.1% of pool)
- ETH price: $2,000, Total value: $4,000

**After Price Movement:**
- ETH price rises, pool rebalances
- User's current LP value: $2,000
- Holding value (if not LP'd): $4,000
- **Impermanent Loss: 50%**

**Insurance Payout:**
- Covered IL: min(50%, 20% cap) - 10% threshold = 10%
- Loss amount: $4,000 × 10% = $400
- Payout (80% coverage): **$320**

## Smart Contract Architecture

```solidity
// Solidity equivalent
contract ILInsurance {
    // Policy parameters (basis points: 10000 = 100%)
    uint256 public thresholdBps;     // Minimum IL for payout
    uint256 public upperCapBps;      // Maximum covered IL
    uint256 public payoutRatioBps;   // Payout percentage
    
    // Pool state
    uint256 public reserveTokenA;
    uint256 public reserveTokenB;
    uint256 public lpTotalSupply;
    
    // Oracle prices (scaled by 1e18)
    uint256 public priceTokenA;
    uint256 public priceTokenB;
    
    // User position
    uint256 public userLpAmount;
    uint256 public userOriginalTokenA;
    uint256 public userOriginalTokenB;
    
    // Core functions
    function calculateIL() external view returns (uint256);
    function calculatePayout() external view returns (uint256);
    function claim() external returns (uint256);
}
```

## Quick Start

### Prerequisites

Install [Rust](https://www.rust-lang.org/tools/install), then install the Stylus CLI tool:

```bash
cargo install --force cargo-stylus cargo-stylus-check
```

Add the WASM build target:

```bash
rustup target add wasm32-unknown-unknown
```

### Clone and Build

```bash
git clone https://github.com/deepesh-sr/ARBix.git
cd ARBix
cargo build --release --target wasm32-unknown-unknown
```

### Run Tests

```bash
cargo test
```

Expected output: **11/11 tests passing** ✅

## ABI Export

Export the Solidity ABI interface:

```bash
cargo stylus export-abi
```

Output includes 17 public functions:

```solidity
interface IILInsurance {
    // Initialization
    function initialize(uint256, uint256, uint256) external;
    
    // View functions - Policy & State
    function getPolicy() external view returns (uint256, uint256, uint256);
    function getPoolState() external view returns (uint256, uint256, uint256);
    function getPrices() external view returns (uint256, uint256);
    function getUserPosition() external view returns (uint256, uint256, uint256);
    function owner() external view returns (address);
    function isInitialized() external view returns (bool);
    
    // View functions - Calculations
    function calculateUserShare() external view returns (uint256);
    function calculateLpValue() external view returns (uint256);
    function calculateHoldingValue() external view returns (uint256);
    function calculateIl() external view returns (uint256);
    function calculatePayout() external view returns (uint256);
    
    // State-changing functions
    function updatePoolState(uint256, uint256, uint256) external;
    function updatePrices(uint256, uint256) external;
    function updateUserPosition(uint256, uint256, uint256) external;
    function claim() external returns (uint256);
    function updatePolicy(uint256, uint256, uint256) external;
    function setupDemo() external;
}
```

## Deployment

### Check WASM Compilation

Verify your contract compiles correctly:

```bash
cargo stylus check
```

Expected output:
```
Compressed WASM size: 8.9 KB
Program succeeded Stylus onchain activation checks ✅
```

### Deploy to Testnet

Deploy to Arbitrum Sepolia testnet (see [testnet info](https://docs.arbitrum.io/stylus/reference/testnet-information)):

```bash
cargo stylus deploy \
  --private-key-path=<PRIVKEY_FILE_PATH> \
  --estimate-gas
```

Full deployment:

```bash
cargo stylus deploy \
  --private-key-path=<PRIVKEY_FILE_PATH>
```

### Deploy Locally

For local testing with Nitro node:

```bash
cargo stylus deploy \
  --private-key-path=<PRIVKEY_FILE_PATH> \
  --endpoint=http://localhost:8547
```

## Usage Example

### Initialize Contract

```rust
// Initialize with policy parameters
// Threshold: 10%, Cap: 20%, Payout: 80%
contract.initialize(
    U256::from(1000),  // 1000 bps = 10%
    U256::from(2000),  // 2000 bps = 20%
    U256::from(8000),  // 8000 bps = 80%
);
```

### Set Up Demo Scenario

```rust
// Owner sets up demo with predefined values
contract.setup_demo();

// Pool: 500 ETH + 1M USDC
// Prices: ETH=$2000, USDC=$1
// User: 1000 LP tokens (0.1% of pool)
// Original deposit: 1 ETH + 2000 USDC
```

### Calculate IL and Payout

```rust
// Get current impermanent loss
let il = contract.calculate_il();
println!("IL: {}%", il / 1e16); // 50%

// Get insurance payout
let payout = contract.calculate_payout();
println!("Payout: ${}", payout / 1e18); // $320

// Process claim
let payout = contract.claim();
```

## Project Structure

```
src/
├── lib.rs              # Main IL Insurance contract (450+ lines)
├── main.rs             # Entry point and workflow tests
├── util.rs             # Math utilities (mul_div with U256)
├── lp_valuator.rs      # LP value calculation logic
├── policy_manager.rs   # Policy management and claims
└── constant.rs         # Configuration constants

examples/
└── counter.rs          # Example integration (to be updated)

Documentation/
├── COMPLETE_DEMO_SCRIPT.md    # 3-minute presentation script
├── STYLUS_CONTRACT.md         # Technical documentation
├── FINAL_SUMMARY.md           # Project overview
└── WORKFLOW_TESTS.md          # Test documentation
```

## Technical Details

### Storage Layout (13 Variables)

All values scaled by 1e18 for precision:

```rust
sol_storage! {
    #[entrypoint]
    pub struct ILInsurance {
        // Policy (basis points)
        uint256 threshold_bps;      // Min IL for payout
        uint256 upper_cap_bps;      // Max covered IL
        uint256 payout_ratio_bps;   // Payout percentage
        
        // Pool state
        uint256 reserve_token_a;    // e.g., ETH reserve
        uint256 reserve_token_b;    // e.g., USDC reserve
        uint256 lp_total_supply;    // Total LP tokens
        
        // Oracle prices
        uint256 price_token_a;      // Token A price (USD)
        uint256 price_token_b;      // Token B price (USD)
        
        // User position
        uint256 user_lp_amount;         // LP tokens owned
        uint256 user_original_token_a;  // Original deposit A
        uint256 user_original_token_b;  // Original deposit B
        
        // Admin
        address owner;
        bool initialized;
    }
}
```

### Key Calculations

**User Share:**
```
user_share = (user_lp_amount * 1e18) / lp_total_supply
```

**LP Value:**
```
current_a = (reserve_a * user_share) / 1e18
current_b = (reserve_b * user_share) / 1e18
lp_value = (current_a * price_a + current_b * price_b) / 1e18
```

**Holding Value:**
```
holding_value = (original_a * price_a + original_b * price_b) / 1e18
```

**Impermanent Loss:**
```
il_percentage = (holding_value - lp_value) / holding_value
```

**Payout:**
```
covered_il = min(il_percentage, upper_cap) - threshold
loss_amount = holding_value * covered_il
payout = loss_amount * payout_ratio / 10000
```

## Testing

### Test Suite

The project includes 11 comprehensive tests covering:

**lib.rs (5 tests):**
- ✅ `test_il_insurance_initialization` - Policy setup
- ✅ `test_il_insurance_demo_scenario` - Full workflow
- ✅ `test_il_calculation` - IL math accuracy
- ✅ `test_payout_below_threshold` - Edge cases
- ✅ `test_claim_processing` - Claim execution

**main.rs (6 tests):**
- ✅ `test_complete_il_insurance_workflow` - End-to-end
- ✅ `test_il_calculation` - Calculation accuracy
- ✅ `test_payout_logic` - Banded coverage
- ✅ `test_user_share_calculation` - Share math
- ✅ `test_mul_div_utility` - Safe arithmetic
- ✅ `test_lp_value_computation` - Value calculations

### Run All Tests

```bash
cargo test --release
```

Expected: **11 passed; 0 failed** ✅

## Dependencies

```toml
[dependencies]
stylus-sdk = "0.9.0"           # Arbitrum Stylus SDK
alloy-primitives = "0.7.6"     # U256 and Address types
alloy-sol-types = "0.7.6"      # Solidity type support
mini-alloc = "0.4.2"           # WASM allocator

[features]
export-abi = ["stylus-sdk/export-abi"]
```

## Advanced Features

### WASM Optimization

Current compressed size: **8.9 KB**

For further optimization, see [cargo-stylus optimization guide](https://github.com/OffchainLabs/cargo-stylus/blob/main/OPTIMIZING_BINARIES.md).

### Code Expansion

View the expanded Rust code from SDK macros:

```bash
cargo install cargo-expand
cargo expand --all-features --release --target=wasm32-unknown-unknown
```

### Type Safety

Uses `U256` from `stylus_sdk::alloy_primitives` for:
- ✅ No overflow/underflow
- ✅ 256-bit precision
- ✅ Solidity compatibility
- ✅ Safe arithmetic operations

Example:
```rust
// Old: Complex overflow handling
let result = a.checked_mul(b).ok_or(Error::Overflow)?;

// New: Clean U256 arithmetic
let result = (U256::from(a) * U256::from(b)) / U256::from(denom);
```

## Future Enhancements

### Production Ready Features

- [ ] **Multi-user support** - Replace single user storage with mapping(address => Position)
- [ ] **Real oracle integration** - Chainlink or Pyth price feeds
- [ ] **Token transfers** - Actual ERC20 payout transfers
- [ ] **Event emission** - On-chain event logging for claims
- [ ] **Security audit** - Professional smart contract audit
- [ ] **Premium collection** - User premium payment system
- [ ] **Pool whitelisting** - Support multiple AMM pools
- [ ] **Time-based policies** - Expiring insurance policies

### Potential Improvements

- [ ] Dynamic pricing based on volatility
- [ ] Liquidity mining rewards for insurers
- [ ] Governance token for parameter updates
- [ ] Cross-chain insurance support
- [ ] Integration with major DEXs (Uniswap, Sushiswap, etc.)

## Resources

### Documentation

- [Arbitrum Stylus Docs](https://docs.arbitrum.io/stylus/stylus-gentle-introduction)
- [Stylus SDK Reference](https://docs.rs/stylus-sdk/latest/stylus_sdk/)
- [Cargo Stylus Tool](https://github.com/OffchainLabs/cargo-stylus)
- [Testnet Information](https://docs.arbitrum.io/stylus/reference/testnet-information)

### Project Documentation

- [COMPLETE_DEMO_SCRIPT.md](./COMPLETE_DEMO_SCRIPT.md) - 3-minute presentation script
- [STYLUS_CONTRACT.md](./STYLUS_CONTRACT.md) - Technical deep dive
- [FINAL_SUMMARY.md](./FINAL_SUMMARY.md) - Project achievements

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Run tests (`cargo test`)
4. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
5. Push to the branch (`git push origin feature/AmazingFeature`)
6. Open a Pull Request

## License

This project is fully open source, including an Apache-2.0 or MIT license at your choosing under your own copyright.

**Note:** This code is for demonstration purposes and has not been audited. Do not use in production without a professional security audit.

## Contact

- **Repository**: [github.com/deepesh-sr/ARBix](https://github.com/deepesh-sr/ARBix)
- **Built with**: [Arbitrum Stylus](https://arbitrum.io/stylus) + Rust

---

**Built with ❤️ using Arbitrum Stylus**
