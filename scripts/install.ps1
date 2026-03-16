# Oxidoc CLI installer for Windows (PowerShell)
#
# Install latest stable:
#   irm https://oxidoc.dev/install.ps1 | iex
#
# Install specific version:
#   $v="0.1.2"; irm https://oxidoc.dev/install.ps1 | iex

param(
    [String]$Version = "latest"
)

$ErrorActionPreference = "Stop"

$Repo = "oxidoc-lab/oxidoc"
$BinName = "oxidoc.exe"

function Install-Oxidoc {
    # Detect architecture
    $Arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
    switch ($Arch) {
        "X64"   { $Target = "windows-x64" }
        "Arm64" { $Target = "windows-arm64" }
        default {
            Write-Error "Unsupported architecture: $Arch"
            return
        }
    }

    # Determine install directory
    $InstallDir = if ($env:OXIDOC_INSTALL_DIR) {
        $env:OXIDOC_INSTALL_DIR
    } else {
        Join-Path $env:USERPROFILE ".oxidoc\bin"
    }

    # Resolve version
    if ($Version -eq "latest") {
        $Version = Get-LatestVersion
        if (-not $Version) {
            Write-Error "Failed to fetch latest version. Check https://github.com/$Repo/releases"
            return
        }
    }

    $VersionNum = $Version -replace "^v", ""

    # Check if already installed
    $ExistingBin = Join-Path $InstallDir $BinName
    if (Test-Path $ExistingBin) {
        try {
            $Current = (& $ExistingBin --version 2>$null) -replace "oxidoc ", ""
            if ($Current -eq $VersionNum) {
                Write-Host "oxidoc $VersionNum is already installed."
                return
            }
            Write-Host "Switching oxidoc $Current -> $VersionNum ($Target)..."
        } catch {
            Write-Host "Installing oxidoc $VersionNum ($Target)..."
        }
    } else {
        Write-Host "Installing oxidoc $VersionNum ($Target)..."
    }

    # Download
    $Url = "https://github.com/$Repo/releases/download/v$VersionNum/oxidoc-$VersionNum-$Target.zip"
    $TempDir = Join-Path ([System.IO.Path]::GetTempPath()) "oxidoc-install-$(Get-Random)"
    $ZipPath = Join-Path $TempDir "oxidoc.zip"

    New-Item -ItemType Directory -Path $TempDir -Force | Out-Null

    Write-Host "Downloading from GitHub Releases..."
    try {
        Invoke-WebRequest -Uri $Url -OutFile $ZipPath -UseBasicParsing
    } catch {
        Write-Error "Download failed. Check the version and try again.`n  $Url"
        return
    }

    # Extract
    Expand-Archive -Path $ZipPath -DestinationPath $TempDir -Force

    $BinPath = Join-Path $TempDir $BinName
    if (-not (Test-Path $BinPath)) {
        # Check subdirectory
        $BinPath = Get-ChildItem -Path $TempDir -Filter $BinName -Recurse | Select-Object -First 1 -ExpandProperty FullName
        if (-not $BinPath) {
            Write-Error "Binary not found in archive"
            return
        }
    }

    # Install
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    Copy-Item -Path $BinPath -Destination (Join-Path $InstallDir $BinName) -Force

    # Clean up
    Remove-Item -Path $TempDir -Recurse -Force -ErrorAction SilentlyContinue

    # Verify
    $InstalledBin = Join-Path $InstallDir $BinName
    Write-Host ""
    Write-Host "Installed oxidoc to $InstalledBin"
    try { & $InstalledBin --version } catch {}

    # Add to PATH if needed
    Add-ToPath $InstallDir

    Write-Host ""
    Write-Host "Get started:"
    Write-Host "  oxidoc init my-docs"
    Write-Host "  cd my-docs"
    Write-Host "  oxidoc dev"
}

function Get-LatestVersion {
    try {
        $Releases = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases?per_page=10" -UseBasicParsing
        foreach ($Release in $Releases) {
            if (-not $Release.prerelease) {
                return $Release.tag_name -replace "^v", ""
            }
        }
    } catch {
        return $null
    }
    return $null
}

function Add-ToPath {
    param([String]$Dir)

    $UserPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
    if ($UserPath -split ";" | Where-Object { $_ -eq $Dir }) {
        return
    }

    [System.Environment]::SetEnvironmentVariable("Path", "$Dir;$UserPath", "User")
    $env:Path = "$Dir;$env:Path"
    Write-Host ""
    Write-Host "Added $Dir to your PATH."
}

Install-Oxidoc
