@echo off
echo Starting Crowdfunding Farmer Contract Deployment...
echo ================================================

set WASM_PATH=C:\Users\hp\Desktop\rev\Revo-Contracts\target\wasm32v1-none\release\crowdfunding_farmer_contract.wasm
set NETWORK=testnet
set IDENTITY=alice

echo.
echo Step 1: Uploading WASM to Stellar %NETWORK%...
echo Running: stellar contract upload --source-account %IDENTITY% --network %NETWORK% --wasm "%WASM_PATH%"
echo.

stellar contract upload --source-account %IDENTITY% --network %NETWORK% --wasm "%WASM_PATH%" > upload_output.txt 2>&1

if %ERRORLEVEL% EQU 0 (
    echo Upload successful!
    type upload_output.txt
    echo.
    echo Please copy the WASM hash from above and run:
    echo stellar contract deploy --source-account %IDENTITY% --network %NETWORK% --wasm-hash [WASM_HASH]
) else (
    echo Upload failed!
    type upload_output.txt
)

pause