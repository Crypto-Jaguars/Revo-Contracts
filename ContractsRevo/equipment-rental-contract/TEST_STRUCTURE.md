# Equipment Rental Contract - Test Structure

## Overview

The equipment-rental-contract now features a comprehensive, modular test structure that provides extensive coverage of the rental system's functionality. The tests are organized into logical modules that correspond to the contract's main areas of functionality.

## Test Structure

```
src/tests/
├── mod.rs           // Module organization
├── utils.rs         // Common test utilities and helpers
├── rental.rs        // Rental agreement creation and management tests
├── availability.rs  // Equipment availability and scheduling tests
└── payment.rs       // Payment processing and validation tests
```

## Running Tests

To run all tests:
```bash
cargo test --lib
```

To run specific test modules:
```bash
cargo test --lib tests::rental
cargo test --lib tests::availability
cargo test --lib tests::payment
```

To run individual tests:
```bash
cargo test --lib test_create_rental_success
```

## Test Coverage

### 📋 Rental Module (rental.rs)
**Total Tests: 14**

- **Agreement Creation Tests:**
  - `test_create_rental_success` - Valid rental creation
  - `test_create_rental_unavailable_equipment` - Equipment unavailable
  - `test_create_rental_equipment_under_maintenance` - Maintenance blocking
  - `test_create_rental_double_booking` - Double booking prevention

- **Lifecycle Management Tests:**
  - `test_confirm_rental_success` - Rental confirmation
  - `test_confirm_rental_not_pending` - Invalid confirmation
  - `test_complete_rental_success` - Rental completion
  - `test_complete_rental_not_active` - Invalid completion
  - `test_cancel_rental_success` - Rental cancellation
  - `test_cancel_rental_already_active` - Invalid cancellation

- **History Tests:**
  - `test_rental_history_by_equipment` - Equipment-based history
  - `test_rental_history_by_user` - User-based history
  - `test_multiple_rentals_same_user` - Multiple rentals per user

- **Integration Tests:**
  - `test_complete_rental_lifecycle` - Full rental lifecycle

### 🔧 Availability Module (availability.rs)
**Total Tests: 15**

- **Equipment Management Tests:**
  - `test_update_equipment_availability` - Availability updates
  - `test_equipment_registration_defaults` - Default values
  - `test_get_nonexistent_equipment` - Missing equipment handling

- **Maintenance Tests:**
  - `test_maintenance_status_transitions` - Status changes
  - `test_maintenance_blocks_rental_creation` - Maintenance blocking rentals
  - `test_maintenance_allows_rental_after_fixed` - Post-maintenance availability

- **Scheduling Conflict Tests:**
  - `test_scheduling_conflict_with_pending_rental` - Pending conflicts
  - `test_scheduling_conflict_with_active_rental` - Active conflicts
  - `test_scheduling_after_rental_completion` - Post-completion scheduling
  - `test_scheduling_after_rental_cancellation` - Post-cancellation scheduling

- **Maintenance History Tests:**
  - `test_maintenance_logging` - Logging maintenance events
  - `test_maintenance_history_filtering` - History filtering

- **High-Volume Tests:**
  - `test_multiple_equipment_availability` - Multiple equipment handling
  - `test_concurrent_rental_attempts` - Concurrent access

### 💰 Payment Module (payment.rs)
**Total Tests: 13**

- **Pricing Calculation Tests:**
  - `test_compute_total_price_success` - Valid price calculation
  - `test_compute_total_price_single_day` - Single day pricing
  - `test_compute_total_price_invalid_dates` - Invalid date handling
  - `test_compute_total_price_nonexistent_equipment` - Missing equipment

- **Price Validation Tests:**
  - `test_validate_price_within_tolerance` - Valid price within tolerance
  - `test_validate_price_exact_match` - Exact price matching
  - `test_validate_price_outside_tolerance_high` - Too high price
  - `test_validate_price_outside_tolerance_low` - Too low price

- **Payment Tracking Tests:**
  - `test_rental_payment_tracking` - Payment amount tracking
  - `test_different_pricing_tiers` - Multiple pricing tiers

- **Edge Case Tests:**
  - `test_zero_day_rental_edge_case` - Zero duration rentals
  - `test_high_value_rental_calculation` - High-value calculations

- **Integration Tests:**
  - `test_pricing_integration_with_rental_flow` - Full pricing flow
  - `test_payment_validation_prevents_invalid_rentals` - Validation integration
  - `test_payment_flow_simulation` - Payment flow simulation

## Key Test Features

### 🛠️ Test Utilities
The `utils.rs` module provides:
- `setup_test()` - Standard test environment setup
- `create_equipment_id()` - Equipment ID generation
- `register_basic_equipment()` - Quick equipment registration
- `advance_time()` - Time manipulation for tests
- `create_standard_rental()` - Standard rental creation

### 🔍 Edge Cases Covered
- Equipment unavailability
- Maintenance status blocking
- Double booking prevention
- Invalid date ranges
- Price tolerance validation
- Scheduling conflicts
- Early completion attempts
- Unauthorized operations

### 🎯 Integration Points
- Payment validation with rental creation
- Maintenance status with rental availability
- Equipment ownership with authorization
- History tracking across operations
- Status transitions with business logic

## Test Results

All **42 tests** pass successfully, providing comprehensive coverage of:
- ✅ Rental agreement creation and management
- ✅ Equipment availability and scheduling
- ✅ Payment processing and validation
- ✅ Error handling and edge cases
- ✅ Business logic enforcement
- ✅ Data integrity and consistency

## Future Integration Points

The test structure is designed to easily accommodate:
- Commodity token contract integration for tokenized payments
- Additional business logic modules
- Performance and load testing
- Integration with external systems

---

*For questions about the test structure or to add new tests, please follow the established patterns in each module.*