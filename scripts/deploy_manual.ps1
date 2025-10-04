# Manual Crowdfunding Farmer Contract Deployment Script for PowerShell
# This script manually deploys the crowdfunding farmer contract to Stellar testnet

param(
    [string]$Network = "testnet",
    [string]$Identity = "alice"
)

Write-Host "🚀 Crowdfunding Farmer Contract Deployment" -ForegroundColor Cyan
Write-Host "=============================================" -ForegroundColor Cyan
Write-Host ""

# Set variables
$PROJECT_ROOT = "C:\Users\hp\Desktop\rev\Revo-Contracts"
$CONTRACT_DIR = "$PROJECT_ROOT\ContractsRevo\crowdfunding-farmer-contract"
$WASM_PATH = "$PROJECT_ROOT\target\wasm32v1-none\release\crowdfunding_farmer_contract.wasm"

Write-Host "Network: $Network" -ForegroundColor Yellow
Write-Host "Identity: $Identity" -ForegroundColor Yellow
Write-Host "Contract Directory: $CONTRACT_DIR" -ForegroundColor Yellow
Write-Host ""

# Step 1: Build the contract
Write-Host "📦 STEP 1: Building Contract" -ForegroundColor Green
Write-Host "=============================" -ForegroundColor Green
Set-Location $CONTRACT_DIR

try {
    Write-Host "Running: stellar contract build --profile release" -ForegroundColor Cyan
    $buildResult = & stellar contract build --profile release 2>&1
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Contract built successfully!" -ForegroundColor Green
        Write-Host $buildResult
    } else {
        Write-Host "❌ Contract build failed!" -ForegroundColor Red
        Write-Host $buildResult
        exit 1
    }
} catch {
    Write-Host "❌ Error building contract: $_" -ForegroundColor Red
    exit 1
}

# Check if WASM file exists
Write-Host ""
Write-Host "🔍 Checking for WASM file..." -ForegroundColor Cyan

$wasmFound = $false
$wasmPaths = @(
    "$PROJECT_ROOT\target\wasm32v1-none\release\crowdfunding_farmer_contract.wasm",
    "$PROJECT_ROOT\target\wasm32-unknown-unknown\release\crowdfunding_farmer_contract.wasm",
    "$CONTRACT_DIR\target\wasm32-unknown-unknown\release\crowdfunding_farmer_contract.wasm"
)

foreach ($path in $wasmPaths) {
    if (Test-Path $path) {
        $WASM_PATH = $path
        $wasmFound = $true
        Write-Host "✅ WASM file found: $WASM_PATH" -ForegroundColor Green
        $fileSize = (Get-Item $WASM_PATH).Length
        Write-Host "📏 File size: $([math]::Round($fileSize/1KB, 2)) KB" -ForegroundColor Cyan
        break
    }
}

if (-not $wasmFound) {
    Write-Host "❌ WASM file not found in any expected location!" -ForegroundColor Red
    Write-Host "Searched locations:" -ForegroundColor Yellow
    foreach ($path in $wasmPaths) {
        Write-Host "  - $path" -ForegroundColor White
    }
    exit 1
}

# Step 2: Upload the contract
Write-Host ""
Write-Host "📤 STEP 2: Uploading Contract" -ForegroundColor Green
Write-Host "==============================" -ForegroundColor Green

try {
    Write-Host "Running: stellar contract upload --source-account $Identity --network $Network --wasm `"$WASM_PATH`"" -ForegroundColor Cyan
    $uploadResult = & stellar contract upload --source-account $Identity --network $Network --wasm $WASM_PATH 2>&1
    
    if ($LASTEXITCODE -eq 0) {
        # Extract WASM hash (should be 64 character hex string)
        $wasmHash = ($uploadResult | Where-Object { $_ -match '^[a-f0-9]{64}$' })[0]
        
        if ($wasmHash) {
            Write-Host "✅ Contract uploaded successfully!" -ForegroundColor Green
            Write-Host "🔑 WASM Hash: $wasmHash" -ForegroundColor Cyan
        } else {
            Write-Host "⚠️ Upload seemed successful but couldn't extract WASM hash" -ForegroundColor Yellow
            Write-Host "Upload output: $uploadResult" -ForegroundColor White
            # Try to extract from the full output
            $wasmHash = Read-Host "Please enter the WASM hash from the output above"
        }
    } else {
        Write-Host "❌ Contract upload failed!" -ForegroundColor Red
        Write-Host $uploadResult
        exit 1
    }
} catch {
    Write-Host "❌ Error uploading contract: $_" -ForegroundColor Red
    exit 1
}

# Step 3: Deploy the contract
Write-Host ""
Write-Host "🚀 STEP 3: Deploying Contract" -ForegroundColor Green
Write-Host "==============================" -ForegroundColor Green

try {
    Write-Host "Running: stellar contract deploy --source-account $Identity --network $Network --wasm-hash $wasmHash" -ForegroundColor Cyan
    $deployResult = & stellar contract deploy --source-account $Identity --network $Network --wasm-hash $wasmHash 2>&1
    
    if ($LASTEXITCODE -eq 0) {
        # Extract contract ID (should start with C and be 56 characters)
        $contractId = ($deployResult | Where-Object { $_ -match '^C[A-Z0-9]{55}$' })[0]
        
        if ($contractId) {
            Write-Host "✅ Contract deployed successfully!" -ForegroundColor Green
            Write-Host "🆔 Contract ID: $contractId" -ForegroundColor Cyan
        } else {
            Write-Host "⚠️ Deploy seemed successful but couldn't extract Contract ID" -ForegroundColor Yellow
            Write-Host "Deploy output: $deployResult" -ForegroundColor White
            $contractId = Read-Host "Please enter the Contract ID from the output above"
        }
    } else {
        Write-Host "❌ Contract deployment failed!" -ForegroundColor Red
        Write-Host $deployResult
        exit 1
    }
} catch {
    Write-Host "❌ Error deploying contract: $_" -ForegroundColor Red
    exit 1
}

# Step 4: Save results
Write-Host ""
Write-Host "💾 STEP 4: Saving Results" -ForegroundColor Green
Write-Host "==========================" -ForegroundColor Green

$timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss UTC"
$logDir = "$CONTRACT_DIR\logs"

# Create logs directory if it doesn't exist
if (-not (Test-Path $logDir)) {
    New-Item -ItemType Directory -Path $logDir -Force | Out-Null
}

# Create results JSON
$results = @{
    contract_name = "crowdfunding-farmer-contract"
    network = $Network
    identity = $Identity
    wasm_hash = $wasmHash
    contract_id = $contractId
    deployment_timestamp = $timestamp
    wasm_path = $WASM_PATH
} | ConvertTo-Json -Depth 3

$resultsFile = "$logDir\deployment_results.json"
$results | Out-File -FilePath $resultsFile -Encoding UTF8

# Create summary file
$summary = @"
Crowdfunding Farmer Contract Deployment Summary
===============================================
Contract: crowdfunding-farmer-contract
Network: $Network
Identity: $Identity
WASM Hash: $wasmHash
Contract ID: $contractId
Deployed: $timestamp
WASM Path: $WASM_PATH
"@

$summaryFile = "$logDir\latest_deployment.txt"
$summary | Out-File -FilePath $summaryFile -Encoding UTF8

Write-Host "✅ Results saved to:" -ForegroundColor Green
Write-Host "  📄 JSON: $resultsFile" -ForegroundColor White
Write-Host "  📄 Summary: $summaryFile" -ForegroundColor White

# Step 5: Display final results
Write-Host ""
Write-Host "🎉 DEPLOYMENT COMPLETED!" -ForegroundColor Green
Write-Host "=========================" -ForegroundColor Green
Write-Host ""
Write-Host "📋 Deployment Details:" -ForegroundColor Cyan
Write-Host "  Contract: crowdfunding-farmer-contract" -ForegroundColor White
Write-Host "  Network: $Network" -ForegroundColor White
Write-Host "  Identity: $Identity" -ForegroundColor White
Write-Host "  WASM Hash: $wasmHash" -ForegroundColor White
Write-Host "  Contract ID: $contractId" -ForegroundColor White
Write-Host ""
Write-Host "🔗 Testnet Explorer:" -ForegroundColor Cyan
Write-Host "  https://testnet.stellar.org/explorer/contract/$contractId" -ForegroundColor Blue
Write-Host ""
Write-Host "📝 Next Steps:" -ForegroundColor Yellow
Write-Host "  1. Verify deployment on Stellar Explorer" -ForegroundColor White
Write-Host "  2. Test contract functionality" -ForegroundColor White
Write-Host "  3. Create and fund campaigns" -ForegroundColor White
Write-Host ""

Write-Host "✅ Crowdfunding Farmer Contract is now live on Stellar $Network!" -ForegroundColor Green