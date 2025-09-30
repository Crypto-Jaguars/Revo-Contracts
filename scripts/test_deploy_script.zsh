#!/bin/zsh

# Test script for deploy_water_management.zsh
# This script validates the deployment script without actually deploying

set -e

echo "🧪 Testing Water Management Deployment Script"
echo "=============================================="

SCRIPT_PATH="./deploy_water_management.zsh"

# Test 1: Help functionality
echo "✅ Test 1: Help functionality"
if $SCRIPT_PATH --help > /dev/null 2>&1; then
    echo "   ✓ Help command works"
else
    echo "   ✗ Help command failed"
    exit 1
fi

# Test 2: Script syntax validation
echo "✅ Test 2: Script syntax validation"
if zsh -n "$SCRIPT_PATH"; then
    echo "   ✓ Script syntax is valid"
else
    echo "   ✗ Script syntax errors found"
    exit 1
fi

# Test 3: Check script is executable
echo "✅ Test 3: Script permissions"
if [[ -x "$SCRIPT_PATH" ]]; then
    echo "   ✓ Script is executable"
else
    echo "   ✗ Script is not executable"
    exit 1
fi

# Test 4: Validate required functions exist
echo "✅ Test 4: Required functions validation"
REQUIRED_FUNCTIONS=(
    "check_prerequisites"
    "validate_parameters" 
    "build_contract"
    "upload_contract"
    "deploy_contract"
    "save_deployment_results"
    "display_results"
    "print_usage"
)

for func in "${REQUIRED_FUNCTIONS[@]}"; do
    if grep -q "^$func()" "$SCRIPT_PATH"; then
        echo "   ✓ Function '$func' exists"
    else
        echo "   ✗ Function '$func' not found"
        exit 1
    fi
done

# Test 5: Check for required variables
echo "✅ Test 5: Required variables validation"
REQUIRED_VARS=(
    "CONTRACT_NAME"
    "CONTRACT_DIR"
    "WASM_PATH"
    "LOG_DIR"
)

for var in "${REQUIRED_VARS[@]}"; do
    if grep -q "^$var=" "$SCRIPT_PATH"; then
        echo "   ✓ Variable '$var' defined"
    else
        echo "   ✗ Variable '$var' not found"
        exit 1
    fi
done

# Test 6: Check for error handling
echo "✅ Test 6: Error handling validation"
if grep -q "set -e" "$SCRIPT_PATH" && grep -q "trap cleanup EXIT" "$SCRIPT_PATH"; then
    echo "   ✓ Error handling mechanisms in place"
else
    echo "   ✗ Error handling mechanisms missing"
    exit 1
fi

# Test 7: Check for logging functionality
echo "✅ Test 7: Logging functionality validation"
if grep -q "DEPLOYMENT_LOG=" "$SCRIPT_PATH" && grep -q "log_with_timestamp" "$SCRIPT_PATH"; then
    echo "   ✓ Logging functionality implemented"
else
    echo "   ✗ Logging functionality missing"
    exit 1
fi

echo ""
echo "🎉 All tests passed! The deployment script is ready to use."
echo ""
echo "Next steps:"
echo "1. Ensure Stellar CLI is installed: cargo install stellar-cli"
echo "2. Install jq: brew install jq"
echo "3. Configure a Stellar profile: stellar config keys add <profile-name>"
echo "4. Run deployment: ./deploy_water_management.zsh testnet"
echo ""
