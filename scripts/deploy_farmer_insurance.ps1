# Farmer Insurance Contract Deployment Script (PowerShell)
# Usage: .\deploy_farmer_insurance.ps1 [network] [profile] [source_identity]
# Example: .\deploy_farmer_insurance.ps1 testnet default alice

param(
    [Parameter(Position=0)]
    [ValidateSet("testnet", "mainnet")]
    [string]$Network = "testnet",
    
    [Parameter(Position=1)]
    [string]$Profile = "default",
    
    [Parameter(Position=2)]
    [string]$SourceIdentity = "alice"
)

# Configuration
$CONTRACT_NAME = "farmer-insurance-contract"
$SCRIPT_DIR = Split-Path -Parent $MyInvocation.MyCommand.Path
$REPO_ROOT = Split-Path -Parent $SCRIPT_DIR
$CONTRACT_DIR = Join-Path $REPO_ROOT "ContractsRevo\farmer-insurance-contract"
$WASM_NAME = "farmer_insurance_contract"
$WASM_PATH = Join-Path $REPO_ROOT "target\wasm32v1-none\release\$WASM_NAME.wasm"
$LOG_DIR = Join-Path $CONTRACT_DIR "logs"
$TIMESTAMP = Get-Date -Format "yyyyMMdd_HHmmss"
$DEPLOYMENT_LOG = Join-Path $LOG_DIR "deployment_$TIMESTAMP.log"

# Color functions
function Write-Info { 
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor Blue
    Add-Content -Path $DEPLOYMENT_LOG -Value "[$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss UTC')] [INFO] $Message"
}

function Write-Success { 
    param([string]$Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor Green
    Add-Content -Path $DEPLOYMENT_LOG -Value "[$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss UTC')] [SUCCESS] $Message"
}

function Write-Warning { 
    param([string]$Message)
    Write-Host "[WARNING] $Message" -ForegroundColor Yellow
    Add-Content -Path $DEPLOYMENT_LOG -Value "[$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss UTC')] [WARNING] $Message"
}

function Write-Error { 
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
    Add-Content -Path $DEPLOYMENT_LOG -Value "[$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss UTC')] [ERROR] $Message"
}

function Write-Header { 
    param([string]$Message)
    Write-Host "================================" -ForegroundColor Cyan
    Write-Host "$Message" -ForegroundColor Cyan
    Write-Host "================================" -ForegroundColor Cyan
    Add-Content -Path $DEPLOYMENT_LOG -Value "[$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss UTC')] === $Message ==="
}

# Function to check prerequisites
function Test-Prerequisites {
    Write-Info "Checking prerequisites..."
    
    # Check if stellar CLI is installed
    try {
        $null = stellar --version
        Write-Success "Stellar CLI found"
    }
    catch {
        Write-Error "Stellar CLI is not installed or not in PATH"
        exit 1
    }
    
    # Check if contract directory exists
    if (-not (Test-Path $CONTRACT_DIR)) {
        Write-Error "Contract directory not found: $CONTRACT_DIR"
        exit 1
    }
    
    Write-Success "Prerequisites check passed"
}

# Function to setup logging
function Initialize-Logging {
    if (-not (Test-Path $LOG_DIR)) {
        New-Item -ItemType Directory -Path $LOG_DIR -Force | Out-Null
        Write-Info "Created log directory: $LOG_DIR"
    }
    
    # Initialize log file
    Add-Content -Path $DEPLOYMENT_LOG -Value "[$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss UTC')] Starting deployment of $CONTRACT_NAME"
    Add-Content -Path $DEPLOYMENT_LOG -Value "[$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss UTC')] Network: $Network"
    Add-Content -Path $DEPLOYMENT_LOG -Value "[$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss UTC')] Profile: $Profile"
    Add-Content -Path $DEPLOYMENT_LOG -Value "[$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss UTC')] Source Identity: $SourceIdentity"
    Add-Content -Path $DEPLOYMENT_LOG -Value "[$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss UTC')] Contract Directory: $CONTRACT_DIR"
    Add-Content -Path $DEPLOYMENT_LOG -Value "[$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss UTC')] WASM Path: $WASM_PATH"
}

# Function to build the contract
function Build-Contract {
    Write-Header "Building Contract"
    
    Set-Location $CONTRACT_DIR
    
    Write-Info "Building $CONTRACT_NAME with profile: $Profile"
    Add-Content -Path $DEPLOYMENT_LOG -Value "[$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss UTC')] Build command: stellar contract build --profile $Profile"
    
    try {
        $buildOutput = stellar contract build --profile $Profile 2>&1
        Add-Content -Path $DEPLOYMENT_LOG -Value $buildOutput
        
        Write-Success "Contract built successfully"
        
        # Verify WASM file exists
        if (Test-Path $WASM_PATH) {
            $wasmSize = (Get-Item $WASM_PATH).Length
            Write-Info "WASM file created: $WASM_PATH ($wasmSize bytes)"
            Add-Content -Path $DEPLOYMENT_LOG -Value "[$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss UTC')] WASM file size: $wasmSize bytes"
        }
        else {
            Write-Error "WASM file not found after build: $WASM_PATH"
            exit 1
        }
    }
    catch {
        Write-Error "Failed to build contract: $_"
        exit 1
    }
}

# Function to upload the contract
function Upload-Contract {
    Write-Header "Uploading Contract"
    
    Write-Info "Uploading WASM to $Network network..."
    Add-Content -Path $DEPLOYMENT_LOG -Value "[$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss UTC')] Upload command: stellar contract upload --source $SourceIdentity --network $Network --wasm $WASM_PATH"
    
    try {
        $uploadOutput = stellar contract upload --source $SourceIdentity --network $Network --wasm $WASM_PATH 2>&1
        Add-Content -Path $DEPLOYMENT_LOG -Value $uploadOutput
        
        # Extract WASM hash from output (last line typically contains the hash)
        $wasmHashLine = ($uploadOutput -split "`n")[-1].Trim()
        
        if ($wasmHashLine -and $wasmHashLine.Length -eq 64) {
            $script:WasmHash = $wasmHashLine
            Write-Success "Contract uploaded successfully"
            Write-Info "WASM Hash: $script:WasmHash"
            Add-Content -Path $DEPLOYMENT_LOG -Value "[$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss UTC')] WASM Hash: $script:WasmHash"
        }
        else {
            Write-Error "Failed to extract WASM hash from upload output"
            Write-Error "Upload output: $uploadOutput"
            exit 1
        }
    }
    catch {
        Write-Error "Failed to upload contract: $_"
        exit 1
    }
}

# Function to deploy the contract
function Deploy-Contract {
    Write-Header "Deploying Contract"
    
    Write-Info "Deploying contract to $Network network..."
    Add-Content -Path $DEPLOYMENT_LOG -Value "[$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss UTC')] Deploy command: stellar contract deploy --source $SourceIdentity --network $Network --wasm $WASM_PATH"
    
    try {
        $deployOutput = stellar contract deploy --source $SourceIdentity --network $Network --wasm $WASM_PATH 2>&1
        Add-Content -Path $DEPLOYMENT_LOG -Value $deployOutput
        
        # Extract contract ID from output (last line typically contains the contract ID)
        $contractIdLine = ($deployOutput -split "`n")[-1].Trim()
        
        if ($contractIdLine -and $contractIdLine.Length -eq 56) {
            $script:ContractId = $contractIdLine
            Write-Success "Contract deployed successfully"
            Write-Info "Contract ID: $script:ContractId"
            Add-Content -Path $DEPLOYMENT_LOG -Value "[$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss UTC')] Contract ID: $script:ContractId"
        }
        else {
            Write-Error "Failed to extract Contract ID from deploy output"
            Write-Error "Deploy output: $deployOutput"
            exit 1
        }
    }
    catch {
        Write-Error "Failed to deploy contract: $_"
        exit 1
    }
}

# Function to save deployment summary
function Save-DeploymentSummary {
    Write-Header "Deployment Summary"
    
    $summaryFile = Join-Path $LOG_DIR "deployment_summary_$TIMESTAMP.json"
    $wasmSize = (Get-Item $WASM_PATH).Length
    
    $summary = @{
        contract_name = $CONTRACT_NAME
        network = $Network
        profile = $Profile
        source_identity = $SourceIdentity
        deployment_timestamp = (Get-Date -Format "yyyy-MM-dd HH:mm:ss UTC")
        wasm_hash = $script:WasmHash
        contract_id = $script:ContractId
        wasm_path = $WASM_PATH
        wasm_size_bytes = $wasmSize
        log_file = $DEPLOYMENT_LOG
    }
    
    $summary | ConvertTo-Json -Depth 3 | Out-File -FilePath $summaryFile -Encoding UTF8
    
    Write-Success "Deployment summary saved to: $summaryFile"
    
    # Display summary
    Write-Host ""
    Write-Info "=== DEPLOYMENT RESULTS ==="
    Write-Info "Contract Name: $CONTRACT_NAME"
    Write-Info "Network: $Network"
    Write-Info "Profile: $Profile"
    Write-Info "Source Identity: $SourceIdentity"
    Write-Info "WASM Hash: $($script:WasmHash)"
    Write-Info "Contract ID: $($script:ContractId)"
    Write-Info "Deployment Time: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss UTC')"
    Write-Info "Log File: $DEPLOYMENT_LOG"
    Write-Info "Summary File: $summaryFile"
    Write-Host ""
    
    # Save to main deployment log
    Add-Content -Path $DEPLOYMENT_LOG -Value "[$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss UTC')] === DEPLOYMENT SUMMARY ==="
    Add-Content -Path $DEPLOYMENT_LOG -Value "[$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss UTC')] WASM Hash: $($script:WasmHash)"
    Add-Content -Path $DEPLOYMENT_LOG -Value "[$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss UTC')] Contract ID: $($script:ContractId)"
    Add-Content -Path $DEPLOYMENT_LOG -Value "[$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss UTC')] Summary saved to: $summaryFile"
}

# Function to verify deployment
function Test-Deployment {
    Write-Header "Verifying Deployment"
    
    Write-Info "Verifying contract deployment..."
    
    try {
        $null = stellar contract invoke --id $script:ContractId --source $SourceIdentity --network $Network -- --help 2>&1
        Write-Success "Contract verification successful - contract is accessible"
        Add-Content -Path $DEPLOYMENT_LOG -Value "[$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss UTC')] Contract verification: SUCCESS"
    }
    catch {
        Write-Warning "Contract verification failed - contract may not be immediately available"
        Add-Content -Path $DEPLOYMENT_LOG -Value "[$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss UTC')] Contract verification: FAILED"
    }
}

# Function to display usage
function Show-Usage {
    Write-Host "Usage: .\deploy_farmer_insurance.ps1 [network] [profile] [source_identity]"
    Write-Host ""
    Write-Host "Parameters:"
    Write-Host "  network         : testnet or mainnet"
    Write-Host "  profile         : build profile (default, release, etc.)"
    Write-Host "  source_identity : stellar identity name for deployment"
    Write-Host ""
    Write-Host "Examples:"
    Write-Host "  .\deploy_farmer_insurance.ps1 testnet default alice"
    Write-Host "  .\deploy_farmer_insurance.ps1 mainnet release production_key"
    Write-Host ""
}

# Main execution
try {
    # Show usage if help requested
    if ($args -contains "-h" -or $args -contains "--help") {
        Show-Usage
        exit 0
    }
    
    Write-Header "Farmer Insurance Contract Deployment"
    Write-Info "Starting deployment process..."
    
    # Check prerequisites
    Test-Prerequisites
    
    # Setup logging
    Initialize-Logging
    
    # Build contract
    Build-Contract
    
    # Upload contract
    Upload-Contract
    
    # Deploy contract
    Deploy-Contract
    
    # Save deployment summary
    Save-DeploymentSummary
    
    # Verify deployment
    Test-Deployment
    
    Write-Success "Deployment completed successfully!"
    Add-Content -Path $DEPLOYMENT_LOG -Value "[$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss UTC')] Deployment process completed successfully"
}
catch {
    Write-Error "Deployment failed: $_"
    Add-Content -Path $DEPLOYMENT_LOG -Value "[$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss UTC')] [ERROR] Deployment failed: $_"
    exit 1
}