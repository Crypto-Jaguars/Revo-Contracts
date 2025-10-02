# Equipment Rental Contract Deployment - PowerShell Wrapper
# Usage: .\deploy_equipment_rental.ps1 [network] [identity]
# Example: .\deploy_equipment_rental.ps1 testnet default

param(
    [Parameter(Mandatory=$true)]
    [ValidateSet("testnet", "mainnet")]
    [string]$Network,
    
    [Parameter(Mandatory=$false)]
    [string]$Identity = "default"
)

# Get script directory
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ZshScript = Join-Path $ScriptDir "deploy_equipment_rental.zsh"

# Colors for PowerShell output
function Write-ColorOutput {
    param(
        [string]$Message,
        [string]$Color = "White"
    )
    Write-Host $Message -ForegroundColor $Color
}

Write-ColorOutput "🔧 Equipment Rental Contract Deployment" "Cyan"
Write-ColorOutput "=========================================" "Cyan"
Write-ColorOutput "Network: $Network" "Yellow"
Write-ColorOutput "Identity: $Identity" "Yellow"
Write-ColorOutput "" 

# Check if WSL is available
if (Get-Command wsl -ErrorAction SilentlyContinue) {
    Write-ColorOutput "✅ WSL detected - running deployment script..." "Green"
    Write-ColorOutput ""
    
    # Convert Windows path to WSL path
    $WslPath = $ZshScript -replace '^([A-Z]):', '/mnt/$1'.ToLower() -replace '\\', '/'
    
    # Run the zsh script in WSL
    wsl zsh "$WslPath" $Network $Identity
    
} elseif (Get-Command bash -ErrorAction SilentlyContinue) {
    Write-ColorOutput "✅ Bash detected - running deployment script..." "Green"
    Write-ColorOutput ""
    
    # Run the zsh script with bash (most zsh scripts work with bash)
    bash $ZshScript $Network $Identity
    
} else {
    Write-ColorOutput "❌ Error: No compatible shell found" "Red"
    Write-ColorOutput ""
    Write-ColorOutput "This script requires either:" "Yellow"
    Write-ColorOutput "  1. WSL (Windows Subsystem for Linux)" "Yellow"
    Write-ColorOutput "  2. Git Bash or similar bash environment" "Yellow"
    Write-ColorOutput ""
    Write-ColorOutput "Installation options:" "Cyan"
    Write-ColorOutput "  • Install WSL: wsl --install" "White"
    Write-ColorOutput "  • Install Git for Windows (includes Git Bash)" "White"
    Write-ColorOutput "  • Install MSYS2 or Cygwin" "White"
    exit 1
}

if ($LASTEXITCODE -eq 0) {
    Write-ColorOutput ""
    Write-ColorOutput "✅ Deployment completed successfully!" "Green"
} else {
    Write-ColorOutput ""
    Write-ColorOutput "❌ Deployment failed with exit code: $LASTEXITCODE" "Red"
    exit $LASTEXITCODE
}