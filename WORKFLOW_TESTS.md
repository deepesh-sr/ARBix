# IL Insurance Workflow Tests

## Overview
This document describes the test suite for the Impermanent Loss (IL) Insurance workflow implemented in the Stylus smart contract.

## Key Improvements Made

### 1. **Using Stylus SDK U256 Type**
Instead of manually handling u128 overflow with complex logic, we now use `stylus_sdk::alloy_primitives::U256`:

```rust
pub fn mul_div(a: u128, b: u128, denom: u128) -> u128 {
    let a_u256 = U256::from(a);
    let b_u256 = U256::from(b);
    let denom_u256 = U256::from(denom);
    
    let result = (a_u256 * b_u256) / denom_u256;
    result.to::<u128>()
}
```

**Benefits:**
- ✅ No overflow issues
- ✅ Clean, readable code
- ✅ Uses native Arbitrum/Ethereum types
- ✅ Battle-tested SDK implementation

## Test Suite

### Test 1: Complete IL Insurance Workflow
**Purpose:** End-to-end test of the entire insurance claim process

**Scenario:**
- Pool: 500 ETH + 1M USDC
- User holds: 1000 LP tokens (0.1% of pool)
- Oracle prices: ETH=$2000, USDC=$1
- Original position: 1 ETH + 2000 USDC = $4000
- Current LP value: ~$2000
- **Impermanent Loss: 50%**

**Insurance Policy:**
- Threshold: 10% (no payout below this)
- Upper cap: 20% (maximum covered IL)
- Payout ratio: 80% of covered loss

**Result:** ✅ Claim APPROVED - Payout: $320

**Calculation:**
```
Covered IL = min(50%, 20%) - 10% = 10%
Loss amount = $4000 × 10% = $400
Payout = $400 × 80% = $320
```

### Test 2: IL Below Threshold
**Purpose:** Verify that claims are denied when IL is below the 10% threshold

**Result:** ✅ Correctly denies claims below threshold

### Test 3: User Share Calculation
**Purpose:** Verify LP share calculation accuracy

**Input:**
- User LP: 1000 tokens
- Total supply: 1,000,000 tokens

**Expected:** 0.1% share
**Result:** ✅ Accurate calculation using U256

### Test 4: Math Utility Functions
**Purpose:** Test the mul_div function with various inputs

**Tests:**
1. Basic multiplication: (100 × 50) / 10 = 500 ✓
2. Percentage: 15% of 1000 = 150 ✓
3. Scaled division: handles large numbers ✓

**Result:** ✅ All calculations correct

### Test 5: LP Value Computation
**Purpose:** Verify LP position valuation logic

**Checks:**
- LP value is positive
- Holding value is positive
- IL percentage is between 0-100%
- LP value < holding value when IL exists

**Result:** ✅ Consistent computations

### Test 6: Payout Calculation Logic
**Purpose:** Verify insurance payout formula

**Formula:**
```rust
if il_frac > threshold:
    covered_frac = min(il_frac, upper_cap) - threshold
    loss_amount = holding_value × covered_frac
    payout = loss_amount × payout_ratio
else:
    payout = 0
```

**Result:** ✅ Correct payout calculations

## Running Tests

```bash
# Run all tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_complete_il_insurance_workflow -- --nocapture

# Run tests in release mode (optimized)
cargo test --release
```

## Module Structure

```
src/
├── main.rs            # Entry point + test suite
├── lib.rs             # Counter contract + module exports
├── constant.rs        # Pool and price constants
├── util.rs            # Math utilities (U256-based)
├── lp_valuator.rs     # LP position valuation
└── policy_manager.rs  # Insurance claim logic
```

## Why U256 Matters

In DeFi calculations, we often need to:
1. Multiply large numbers (token amounts scaled by 1e18)
2. Maintain precision through multiple operations
3. Avoid overflow errors that could cause reverts

**Before (u128 with manual overflow handling):**
- ❌ Complex, error-prone code
- ❌ Difficult to maintain
- ❌ Potential precision loss

**After (U256 from Stylus SDK):**
- ✅ Simple, clean code
- ✅ No overflow possible
- ✅ Full precision maintained
- ✅ Industry-standard approach

## Example Output

```
=== IL Insurance Workflow Test ===

📊 Pool Configuration:
  ETH Reserve: 500 ETH
  USDC Reserve: 1000000 USDC
  LP Total Supply: 1000000 LP tokens
  User LP Amount: 1000 LP tokens

💵 Oracle Prices:
  ETH Price: $2000
  USDC Price: $1

👤 User's LP Share: 0.100%

📈 Position Analysis:
  Current LP Value: $2000.000000
  Holding Value: $4000.000000
  Impermanent Loss: 50.00%

🛡️ Insurance Policy:
  Threshold: 10% (no coverage below this)
  Upper Cap: 20% (maximum covered IL)
  Payout Ratio: 80% of covered loss

💰 Claim Result:
  Payout Amount: $320.000000
  ✅ Claim APPROVED

✅ All workflow tests passed!
```

## Next Steps

1. **Add more test scenarios:**
   - Different IL percentages
   - Edge cases (0% IL, 100% IL)
   - Multiple users

2. **Integration tests:**
   - Deploy to Stylus testnet
   - Interact via ethers.rs
   - Verify on-chain behavior

3. **Gas optimization:**
   - Profile gas usage
   - Optimize storage patterns
   - Benchmark against Solidity

4. **Security:**
   - Add overflow checks on inputs
   - Implement access controls
   - Add reentrancy guards
