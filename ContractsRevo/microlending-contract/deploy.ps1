# Microlending Contract Deployment Script (PowerShell)
# This script automates the deployment process for the microlending contract

param(
    [Parameter(Mandatory=$true)]
    [ValidateSet("build", "testnet", "mainnet", "test")]
    [string]$Command,
    
    [Parameter(Mandatory=$false)]
    [string]$SourceAccount,
    
    [Parameter(Mandatory=$false)]
    [string]$ContractId,
    
    [Parameter(Mandatory=$false)]
    [string]$Network
)

# Function to print colored output
function Write-Status {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor Blue
}

function Write-Success {
    param([string]$Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor Green
}

function Write-Warning {
    param([string]$Message)
    Write-Host "[WARNING] $Message" -ForegroundColor Yellow
}

function Write-Error {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

# Check if required tools are installed
function Test-Prerequisites {
    Write-Status "Checking prerequisites..."
    
    if (-not (Get-Command soroban -ErrorAction SilentlyContinue)) {
        Write-Error "Soroban CLI is not installed. Please install it first:"
        Write-Host "cargo install soroban-cli"
        exit 1
    }
    
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-Error "Cargo is not installed. Please install Rust first."
        exit 1
    }
    
    Write-Success "Prerequisites check passed"
}

# Build the contract
function Build-Contract {
    Write-Status "Building the microlending contract..."
    
    # Build with Cargo
    cargo build --target wasm32-unknown-unknown --release
    
    # Build with Soroban CLI
    soroban contract build
    
    Write-Success "Contract built successfully"
}

# Deploy to testnet
function Deploy-Testnet {
    param([string]$SourceAccount)
    
    if ([string]::IsNullOrEmpty($SourceAccount)) {
        Write-Error "Source account is required for deployment"
        Write-Host "Usage: .\deploy.ps1 testnet -SourceAccount <account>"
        exit 1
    }
    
    Write-Status "Deploying to Stellar testnet..."
    Write-Warning "Make sure your account has sufficient XLM for deployment fees"
    
    # Deploy the contract
    $contractId = soroban contract deploy --wasm target/wasm32-unknown-unknown/release/micro_lending.wasm --network testnet --source $SourceAccount
    
    Write-Success "Contract deployed successfully!"
    Write-Host "Contract ID: $contractId"
    
    # Update DEPLOYMENTS.md
    Update-DeploymentsFile $contractId "testnet"
    
    return $contractId
}

# Deploy to mainnet
function Deploy-Mainnet {
    param([string]$SourceAccount)
    
    if ([string]::IsNullOrEmpty($SourceAccount)) {
        Write-Error "Source account is required for deployment"
        Write-Host "Usage: .\deploy.ps1 mainnet -SourceAccount <account>"
        exit 1
    }
    
    Write-Warning "You are about to deploy to MAINNET!"
    Write-Warning "This will cost real XLM and cannot be undone."
    $confirm = Read-Host "Are you sure you want to continue? (yes/no)"
    
    if ($confirm -ne "yes") {
        Write-Status "Deployment cancelled"
        exit 0
    }
    
    Write-Status "Deploying to Stellar mainnet..."
    
    # Deploy the contract
    $contractId = soroban contract deploy --wasm target/wasm32-unknown-unknown/release/micro_lending.wasm --network mainnet --source $SourceAccount
    
    Write-Success "Contract deployed successfully to mainnet!"
    Write-Host "Contract ID: $contractId"
    
    # Update DEPLOYMENTS.md
    Update-DeploymentsFile $contractId "mainnet"
    
    return $contractId
}

# Update DEPLOYMENTS.md file
function Update-DeploymentsFile {
    param([string]$ContractId, [string]$Network)
    
    $date = Get-Date -Format "yyyy-MM-dd HH:mm:ss UTC"
    
    Write-Status "Updating DEPLOYMENTS.md..."
    
    # Update the deployments file
    $content = Get-Content "DEPLOYMENTS.md"
    $updatedContent = $content -replace "\| TBD \| $Network \| TBD \| pending \|", "| $ContractId | $Network | $date | deployed |"
    $updatedContent | Set-Content "DEPLOYMENTS.md"
    
    Write-Success "DEPLOYMENTS.md updated with contract ID: $ContractId"
}

# Test the deployed contract
function Test-Contract {
    param([string]$ContractId, [string]$Network, [string]$SourceAccount)
    
    if ([string]::IsNullOrEmpty($ContractId) -or [string]::IsNullOrEmpty($Network) -or [string]::IsNullOrEmpty($SourceAccount)) {
        Write-Error "Contract ID, network, and source account are required for testing"
        return
    }
    
    Write-Status "Testing deployed contract..."
    
    # Test basic contract functionality
    Write-Status "Testing contract initialization..."
    
    # Note: This is a placeholder - actual testing would require a token address
    Write-Warning "Manual testing required:"
    Write-Host "1. Initialize the contract with a token address:"
    Write-Host "   soroban contract invoke --id $ContractId --source $SourceAccount --network $Network -- initialize --token_address <TOKEN_ADDRESS>"
    Write-Host ""
    Write-Host "2. Create a test loan request:"
    Write-Host "   soroban contract invoke --id $ContractId --source $SourceAccount --network $Network -- create_loan_request --borrower <BORROWER_ADDRESS> --amount 1000 --purpose 'Test loan' --duration_days 30 --interest_rate 1000 --collateral '{\"asset_type\": \"Test\", \"estimated_value\": 1500, \"verification_data\": \"0x0000000000000000000000000000000000000000000000000000000000000000\"}'"
}

# Main execution
switch ($Command) {
    "build" {
        Test-Prerequisites
        Build-Contract
    }
    "testnet" {
        Test-Prerequisites
        Build-Contract
        Deploy-Testnet $SourceAccount
    }
    "mainnet" {
        Test-Prerequisites
        Build-Contract
        Deploy-Mainnet $SourceAccount
    }
    "test" {
        Test-Contract $ContractId $Network $SourceAccount
    }
    default {
        Write-Host "Usage: .\deploy.ps1 {build|testnet|mainnet|test} [parameters]"
        Write-Host ""
        Write-Host "Commands:"
        Write-Host "  build                    - Build the contract"
        Write-Host "  testnet -SourceAccount <account> - Deploy to testnet"
        Write-Host "  mainnet -SourceAccount <account> - Deploy to mainnet"
        Write-Host "  test -ContractId <id> -Network <network> -SourceAccount <account> - Test deployed contract"
        Write-Host ""
        Write-Host "Examples:"
        Write-Host "  .\deploy.ps1 build"
        Write-Host "  .\deploy.ps1 testnet -SourceAccount GBZXN7PIRZGNWCXXFYU7KYWXX4BXZUYHZO5QUEMKRHLUVLYN53WVFG3E"
        Write-Host "  .\deploy.ps1 test -ContractId CACDYF3CYMJEJTIVFESQYZTN67GO2R5D5IUABTCUG3HXQSRXCSOROBAN -Network testnet -SourceAccount GBZXN7PIRZGNWCXXFYU7KYWXX4BXZUYHZO5QUEMKRHLUVLYN53WVFG3E"
        exit 1
    }
}
