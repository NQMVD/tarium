# Build Tarium with Embedded GitHub App Credentials
# This script builds Tarium with GitHub App credentials embedded in the binary

param(
    [Parameter(Mandatory=$true, HelpMessage="Your GitHub App ID")]
    [string]$AppId,

    [Parameter(Mandatory=$true, HelpMessage="Your GitHub App Installation ID")]
    [string]$InstallationId,

    [Parameter(Mandatory=$true, HelpMessage="Path to your private key .pem file")]
    [string]$PrivateKeyPath,

    [Parameter(HelpMessage="Build target (e.g., x86_64-pc-windows-msvc)")]
    [string]$Target = "",

    [Parameter(HelpMessage="Build profile (debug or release)")]
    [ValidateSet("debug", "release")]
    [string]$Profile = "release",

    [Parameter(HelpMessage="Additional cargo build flags")]
    [string]$ExtraFlags = ""
)

Write-Host "Building Tarium with embedded GitHub App credentials..." -ForegroundColor Blue

# Validate parameters
if ([string]::IsNullOrWhiteSpace($AppId)) {
    Write-Error "App ID cannot be empty"
    exit 1
}

if ([string]::IsNullOrWhiteSpace($InstallationId)) {
    Write-Error "Installation ID cannot be empty"
    exit 1
}

if ([string]::IsNullOrWhiteSpace($PrivateKeyPath)) {
    Write-Error "Private key path cannot be empty"
    exit 1
}

# Check if private key file exists
if (-not (Test-Path $PrivateKeyPath)) {
    Write-Error "Private key file not found at: $PrivateKeyPath"
    exit 1
}

# Resolve full path
$FullPrivateKeyPath = Resolve-Path $PrivateKeyPath

# Validate App ID is numeric
if (-not ($AppId -match '^\d+$')) {
    Write-Error "App ID must be numeric: $AppId"
    exit 1
}

# Validate Installation ID is numeric
if (-not ($InstallationId -match '^\d+$')) {
    Write-Error "Installation ID must be numeric: $InstallationId"
    exit 1
}

Write-Host "App ID: $AppId"
Write-Host "Installation ID: $InstallationId"
Write-Host "Private Key: $FullPrivateKeyPath"
Write-Host "Profile: $Profile"

# Set environment variables for the build
$env:TARIUM_EMBED_CREDENTIALS = "1"
$env:TARIUM_EMBED_GITHUB_APP_ID = $AppId
$env:TARIUM_EMBED_GITHUB_INSTALLATION_ID = $InstallationId
$env:TARIUM_EMBED_GITHUB_PRIVATE_KEY_PATH = $FullPrivateKeyPath

# Build the cargo command
$cargoArgs = @("build")

if ($Profile -eq "release") {
    $cargoArgs += "--release"
}

if ($Target) {
    $cargoArgs += "--target", $Target
    Write-Host "Target: $Target"
}

if ($ExtraFlags) {
    $cargoArgs += $ExtraFlags.Split(' ')
    Write-Host "Extra flags: $ExtraFlags"
}

Write-Host ""
Write-Host "Running cargo build..."
Write-Host "Command: cargo $($cargoArgs -join ' ')"

# Run the build
try {
    & cargo @cargoArgs

    if ($LASTEXITCODE -eq 0) {
        Write-Host ""
        Write-Host "Build completed successfully!" -ForegroundColor Green

        # Determine binary path
        $binaryName = "tarium.exe"
        if ($Target) {
            $binaryPath = "target\$Target\$Profile\$binaryName"
        } else {
            $binaryPath = "target\$Profile\$binaryName"
        }

        if (Test-Path $binaryPath) {
            $fileSize = (Get-Item $binaryPath).Length
            $fileSizeMB = [math]::Round($fileSize / 1MB, 2)

            Write-Host "Binary created: $binaryPath"
            Write-Host "Size: $fileSizeMB MB"
            Write-Host ""
            Write-Host "The binary now contains embedded GitHub App credentials"
            Write-Host "Users can run it without any authentication setup"
            Write-Host ""
            Write-Host "Test the binary with:"
            Write-Host "  $binaryPath auth status"
        } else {
            Write-Warning "Binary not found at expected path: $binaryPath"
        }
    } else {
        Write-Error "Build failed with exit code: $LASTEXITCODE"
        exit $LASTEXITCODE
    }
}
catch {
    Write-Error "Build failed: $_"
    exit 1
}
finally {
    # Clean up environment variables
    Remove-Item env:TARIUM_EMBED_CREDENTIALS -ErrorAction SilentlyContinue
    Remove-Item env:TARIUM_EMBED_GITHUB_APP_ID -ErrorAction SilentlyContinue
    Remove-Item env:TARIUM_EMBED_GITHUB_INSTALLATION_ID -ErrorAction SilentlyContinue
    Remove-Item env:TARIUM_EMBED_GITHUB_PRIVATE_KEY_PATH -ErrorAction SilentlyContinue
}

Write-Host ""
Write-Host "Security note:" -ForegroundColor Yellow
Write-Host "  The private key is now embedded in the binary"
Write-Host "  Treat this binary as sensitive - don't share in public repositories"
Write-Host "  For open source distribution, use a separate GitHub App"
