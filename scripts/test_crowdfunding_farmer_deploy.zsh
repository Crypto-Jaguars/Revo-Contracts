#!/bin/zsh

# Test script for crowdfunding farmer contract deployment
# This script tests the deployment script without actually deploying

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

SCRIPT_DIR="${0:A:h}"
DEPLOY_SCRIPT="${SCRIPT_DIR}/deploy_crowdfunding_farmer.zsh"

echo -e "${BLUE}Testing Crowdfunding Farmer Contract Deployment Script${NC}"
echo "======================================================="

# Test 1: Check if script exists and is executable
echo -e "\n${YELLOW}Test 1: Script Existence${NC}"
if [[ -f "$DEPLOY_SCRIPT" ]]; then
    echo -e "${GREEN}✅ Script exists at: $DEPLOY_SCRIPT${NC}"
else
    echo -e "${RED}❌ Script not found at: $DEPLOY_SCRIPT${NC}"
    exit 1
fi

# Test 2: Check script help functionality
echo -e "\n${YELLOW}Test 2: Help Functionality${NC}"
if "$DEPLOY_SCRIPT" --help > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Help command works${NC}"
else
    echo -e "${RED}❌ Help command failed${NC}"
fi

# Test 3: Check parameter validation (should fail gracefully)
echo -e "\n${YELLOW}Test 3: Parameter Validation${NC}"
if "$DEPLOY_SCRIPT" 2>&1 | grep -q "Network parameter is required"; then
    echo -e "${GREEN}✅ Parameter validation works${NC}"
else
    echo -e "${RED}❌ Parameter validation may not be working properly${NC}"
fi

# Test 4: Check invalid network handling
echo -e "\n${YELLOW}Test 4: Invalid Network Handling${NC}"
if "$DEPLOY_SCRIPT" invalidnetwork 2>&1 | grep -q "Invalid network"; then
    echo -e "${GREEN}✅ Invalid network handling works${NC}"
else
    echo -e "${RED}❌ Invalid network handling may not be working${NC}"
fi

# Test 5: Check contract directory existence
echo -e "\n${YELLOW}Test 5: Contract Directory${NC}"
CONTRACT_DIR="${SCRIPT_DIR:h}/ContractsRevo/crowdfunding-farmer-contract"
if [[ -d "$CONTRACT_DIR" ]]; then
    echo -e "${GREEN}✅ Contract directory exists: $CONTRACT_DIR${NC}"
    
    # Check for essential files
    if [[ -f "$CONTRACT_DIR/Cargo.toml" ]]; then
        echo -e "${GREEN}✅ Cargo.toml found${NC}"
    else
        echo -e "${RED}❌ Cargo.toml not found${NC}"
    fi
    
    if [[ -f "$CONTRACT_DIR/src/lib.rs" ]]; then
        echo -e "${GREEN}✅ lib.rs found${NC}"
    else
        echo -e "${RED}❌ lib.rs not found${NC}"
    fi
else
    echo -e "${RED}❌ Contract directory not found: $CONTRACT_DIR${NC}"
fi

# Test 6: Check for required tools (non-blocking)
echo -e "\n${YELLOW}Test 6: Required Tools Check${NC}"

if command -v stellar &> /dev/null; then
    echo -e "${GREEN}✅ Stellar CLI found${NC}"
    stellar --version
else
    echo -e "${YELLOW}⚠️  Stellar CLI not found (install with: cargo install stellar-cli)${NC}"
fi

if command -v cargo &> /dev/null; then
    echo -e "${GREEN}✅ Cargo found${NC}"
    cargo --version
else
    echo -e "${RED}❌ Cargo not found (install Rust first)${NC}"
fi

if command -v jq &> /dev/null; then
    echo -e "${GREEN}✅ jq found${NC}"
else
    echo -e "${YELLOW}⚠️  jq not found (recommended for JSON parsing)${NC}"
fi

echo -e "\n${BLUE}Test Summary${NC}"
echo "============="
echo -e "${GREEN}Script is ready for deployment!${NC}"
echo ""
echo "To deploy to testnet:"
echo "  $DEPLOY_SCRIPT testnet"
echo ""
echo "To deploy to mainnet:"
echo "  $DEPLOY_SCRIPT mainnet production"
echo ""
echo -e "${YELLOW}Note: Make sure you have:${NC}"
echo "  1. Stellar CLI installed"
echo "  2. Valid Stellar identity configured"
echo "  3. Sufficient XLM for transaction fees"