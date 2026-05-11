<#
.SYNOPSIS
    Robust Universal Installer for metapak (Windows)
.DESCRIPTION
    Clones the repository, installs Rust if necessary, builds the release binary,
    copies it to LocalAppData, and safely injects it into the User PATH.
#>

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

# Paths
$RepoUrl = "https://github.com/sreevarshan-xenoz/metapak.git"
$InstallDir = Join-Path $env:LOCALAPPDATA "metapak\bin"
$ConfigDir = Join-Path $env:APPDATA "metapak"
$TempDirName = "metapak-install-" + [guid]::NewGuid().ToString().Substring(0,8)
$TempDir = Join-Path $env:TEMP $TempDirName

Write-Host "=== metapak Installer (Windows) ===" -ForegroundColor Cyan

# 1. Dependency Validation
Write-Host "[1/5] Checking dependencies..." -ForegroundColor Green

if (-not (Get-Command git -ErrorAction Ignore)) {
    Write-Host "Error: 'git' is required but not found in PATH." -ForegroundColor Red
    Write-Host "Please install Git (e.g., 'winget install Git.Git') and try again." -ForegroundColor Yellow
    exit 1
}

if (-not (Get-Command cargo -ErrorAction Ignore)) {
    Write-Host "Rust/Cargo not found. Downloading rustup-init..." -ForegroundColor Yellow
    $RustupInit = Join-Path $env:TEMP "rustup-init.exe"
    Invoke-WebRequest -Uri "https://win.rustup.rs" -OutFile $RustupInit
    
    Write-Host "Running rustup installer..." -ForegroundColor Cyan
    Start-Process -FilePath $RustupInit -ArgumentList "-y", "--default-host", "x86_64-pc-windows-msvc" -Wait -NoNewWindow
    Remove-Item $RustupInit -Force
    
    # Reload environment block
    Write-Host "Reloading environment variables..." -ForegroundColor Cyan
    foreach ($level in "Machine", "User") {
        [Environment]::GetEnvironmentVariables($level).GetEnumerator() | ForEach-Object {
            [Environment]::SetEnvironmentVariable($_.Name, $_.Value, "Process")
        }
    }
    
    if (-not (Get-Command cargo -ErrorAction Ignore)) {
        Write-Host "Failed to find cargo after installation. You may need to restart your terminal." -ForegroundColor Red
        exit 1
    }
} else {
    $cargoVer = (cargo --version).Trim()
    Write-Host "Found Rust: $cargoVer"
}

try {
    # 2. Clone Repository
    Write-Host "[2/5] Downloading source code..." -ForegroundColor Green
    git clone --depth 1 $RepoUrl $TempDir | Out-Null
    Set-Location $TempDir

    # 3. Build Release
    Write-Host "[3/5] Building release binary (this may take a few minutes)..." -ForegroundColor Green
    cargo build --release

    # 4. Install Binary & Config
    Write-Host "[4/5] Installing binary and configuration..." -ForegroundColor Green
    if (-not (Test-Path $InstallDir)) {
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    }
    
    # Idempotence check
    $TargetExe = Join-Path $InstallDir "metapak.exe"
    if (Test-Path $TargetExe) {
        Write-Host "metapak is already installed. Overwriting binary..." -ForegroundColor Yellow
    }
    
    Copy-Item "target\release\metapak.exe" -Destination $TargetExe -Force
    
    if (-not (Test-Path $ConfigDir)) {
        New-Item -ItemType Directory -Path $ConfigDir -Force | Out-Null
    }
    
    $TargetConfig = Join-Path $ConfigDir "config.toml"
    if (-not (Test-Path $TargetConfig)) {
        Copy-Item "config.example.toml" -Destination $TargetConfig -Force
        Write-Host "Created default config at $TargetConfig"
    } else {
        Write-Host "Config already exists. Skipping..."
    }

    # 5. PATH Injection
    Write-Host "[5/5] Finalizing installation..." -ForegroundColor Green
    
    $UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    $PathArray = $UserPath -split ';'
    
    if ($InstallDir -notin $PathArray) {
        Write-Host ("Adding " + $InstallDir + " to User PATH...") -ForegroundColor Yellow
        $NewPath = $UserPath + ";" + $InstallDir
        [Environment]::SetEnvironmentVariable("PATH", $NewPath, "User")
        
        # Inject into current process so it works immediately
        $Env:PATH += ";" + $InstallDir
        Write-Host "PATH updated successfully. You may need to restart your terminal for changes to fully apply." -ForegroundColor Cyan
    }
    
    Write-Host ""
    Write-Host "[OK] metapak Installation Complete!" -ForegroundColor Green
    Write-Host "Run 'metapak' to get started." -ForegroundColor Cyan

} catch {
    $errMsg = $_.Exception.Message
    Write-Host ""
    Write-Host ("Installation failed: " + $errMsg) -ForegroundColor Red
    exit 1
} finally {
    # Cleanup
    if (Test-Path $TempDir) {
        Set-Location $env:TEMP # Move out of temp dir to allow deletion
        Remove-Item -Path $TempDir -Recurse -Force -ErrorAction Ignore
    }
}
