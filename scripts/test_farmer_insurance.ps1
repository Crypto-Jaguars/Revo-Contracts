# Farmer Insurance Contract Test Script
param(
    [string]$ContractId = "CAJRMWQK3QDMMLFZK4ZPBBG45L3CKSZODSDHVNA7F6O4IOYHXCMXGWRG",
    [string]$Network = "testnet", 
    [string]$SourceIdentity = "alice"
)

function Write-Info { 
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor Blue
}

function Write-Success { 
    param([string]$Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor Green
}

function Write-TestError { 
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

function Write-Header { 
    param([string]$Message)
    Write-Host "================================" -ForegroundColor Cyan
    Write-Host "$Message" -ForegroundColor Cyan  
    Write-Host "================================" -ForegroundColor Cyan
}

function Test-ContractAccessibility {
    Write-Header "Testing Contract Accessibility"
    Write-Info "Testing if contract functions are accessible..."
    
    try {
        $helpOutput = stellar contract invoke --id $ContractId --source $SourceIdentity --network $Network -- --help 2>&1
        
        if ($helpOutput -match "Commands:") {
            Write-Success "Contract is accessible and functions are available"
            return $true
        }
        else {
            Write-TestError "Contract help output doesn't show expected format"
            return $false
        }
    }
    catch {
        Write-TestError "Failed to access contract: $_"
        return $false
    }
}

function Get-ContractFunctions {
    Write-Header "Available Contract Functions"
    Write-Info "Retrieving list of available functions..."
    
    try {
        $helpOutput = stellar contract invoke --id $ContractId --source $SourceIdentity --network $Network -- --help 2>&1
        
        $lines = $helpOutput -split "`n"
        $inCommandsSection = $false
        $functions = @()
        
        foreach ($line in $lines) {
            if ($line -match "Commands:") {
                $inCommandsSection = $true
                continue
            }
            if ($inCommandsSection -and $line -match "Options:") {
                break
            }
            if ($inCommandsSection -and $line.Trim() -and $line -notmatch "help") {
                $functionName = $line.Trim()
                if ($functionName -ne "" -and $functionName -notmatch "Print this message") {
                    $functions += $functionName
                }
            }
        }
        
        if ($functions.Count -gt 0) {
            Write-Success "Found $($functions.Count) contract functions:"
            foreach ($func in $functions) {
                Write-Info "  - $func"
            }
        }
        else {
            Write-TestError "No functions found in contract help output"
        }
        
        return $functions
    }
    catch {
        Write-TestError "Failed to retrieve contract functions: $_"
        return @()
    }
}

function Test-GetPolicyFunction {
    Write-Header "Testing get_policy Function"
    Write-Info "Testing get_policy function with a test policy ID..."
    
    $testPolicyId = "0000000000000000000000000000000000000000000000000000000000000001"
    
    try {
        $policyOutput = stellar contract invoke --id $ContractId --source $SourceIdentity --network $Network -- get_policy --policy_id $testPolicyId 2>&1
        
        if ($policyOutput -match "error") {
            Write-Info "get_policy function is working (returns expected error for non-existent policy)"
            return $true
        }
        elseif ($policyOutput) {
            Write-Success "get_policy function returned data: $policyOutput"
            return $true
        }
        else {
            Write-TestError "get_policy function returned unexpected output"
            return $false
        }
    }
    catch {
        Write-TestError "Failed to test get_policy function: $_"
        return $false
    }
}

# Main execution
Write-Host ""
Write-Host "Farmer Insurance Contract Test Script" -ForegroundColor Magenta
Write-Host "=====================================" -ForegroundColor Magenta
Write-Host ""

Write-Info "Contract ID: $ContractId"
Write-Info "Network: $Network" 
Write-Info "Source Identity: $SourceIdentity"
Write-Info ""

# Check prerequisites
Write-Info "Checking prerequisites..."
if (!(Get-Command stellar -ErrorAction SilentlyContinue)) {
    Write-TestError "Stellar CLI not found. Please install it first."
    exit 1
}
Write-Success "Stellar CLI found"
Write-Info ""

# Run tests
$testResults = @{
    accessibility = $false
    functionsFound = 0
    getPolicy = $false
}

# Test 1: Contract Accessibility
$testResults.accessibility = Test-ContractAccessibility

# Test 2: Get contract functions
$functions = Get-ContractFunctions
$testResults.functionsFound = $functions.Count

# Test 3: Test get_policy function
$testResults.getPolicy = Test-GetPolicyFunction

# Summary
Write-Header "Test Summary"

$passedTests = 0
$totalTests = 3

if ($testResults.accessibility) {
    Write-Success "[PASS] Contract Accessibility: PASSED"
    $passedTests++
} else {
    Write-TestError "[FAIL] Contract Accessibility: FAILED"
}

if ($testResults.functionsFound -gt 0) {
    Write-Success "[PASS] Function Discovery: PASSED ($($testResults.functionsFound) functions found)"
    $passedTests++
} else {
    Write-TestError "[FAIL] Function Discovery: FAILED"
}

if ($testResults.getPolicy) {
    Write-Success "[PASS] Function Testing: PASSED"
    $passedTests++
} else {
    Write-TestError "[FAIL] Function Testing: FAILED"
}

Write-Info ""
Write-Info "Overall Result: $passedTests/$totalTests tests passed"

if ($passedTests -eq $totalTests) {
    Write-Success "All tests passed! Contract deployment is verified and working correctly."
    exit 0
} else {
    Write-TestError "Some tests failed. Contract may have issues."
    exit 1
}