<#
.SYNOPSIS
    Packs the already-built Radiocove release exe (src-tauri\target\release) into a
    local .msix, without the MSIX Packaging Tool GUI. Does not build anything itself -
    run `cargo build --release` / `npm run tauri build` yourself first.

.PARAMETER Arch
    "x64" or "arm64". Only affects the manifest's ProcessorArchitecture and output
    filename - it does not cross-compile. Default: x64.

.PARAMETER Sign
    Sign the package with a local self-signed test certificate (created once, reused after).

.PARAMETER Install
    Install the resulting package on this machine after packing (Add-AppxPackage).

.EXAMPLE
    .\scripts\build-msix.ps1 -Sign -Install
#>
param(
    [ValidateSet("x64", "arm64")]
    [string]$Arch = "x64",
    [switch]$Sign,
    [switch]$Install
)

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $PSScriptRoot
$TauriDir = Join-Path $RepoRoot "src-tauri"
$MsixDir = Join-Path $RepoRoot "scripts\msix"
$CertDir = Join-Path $MsixDir ".cert"
$StagingDir = Join-Path $TauriDir "target\msix-staging"
$OutDir = Join-Path $TauriDir "target\msix-out"

$ArchMsix = @{ x64 = "x64"; arm64 = "arm64" }

# PackageFamilyName's "PublisherId" segment: SHA-256 of the UTF-16LE Publisher string,
# first 64 bits, encoded 5 bits at a time in this base32-ish alphabet. Same algorithm
# Windows itself uses, so this matches whatever the real Store package would be named.
function Get-PublisherId([string]$Publisher) {
    $bytes = [System.Text.Encoding]::Unicode.GetBytes($Publisher)
    $hash = [System.Security.Cryptography.SHA256]::Create().ComputeHash($bytes)
    $alphabet = "0123456789abcdefghjkmnpqrstvwxyz"
    $bits = ($hash[0..7] | ForEach-Object { [Convert]::ToString($_, 2).PadLeft(8, '0') }) -join ''
    $bits += '0'
    $result = ""
    for ($i = 0; $i -lt 13; $i++) {
        $index = [Convert]::ToInt32($bits.Substring($i * 5, 5), 2)
        $result += $alphabet[$index]
    }
    return $result
}

function Find-SdkTool([string]$Name) {
    $cmd = Get-Command $Name -ErrorAction SilentlyContinue
    if ($cmd) { return $cmd.Source }
    $found = Get-ChildItem "C:\Program Files (x86)\Windows Kits\10\bin" -Recurse -Filter $Name -ErrorAction SilentlyContinue |
        Where-Object { $_.FullName -match "\\x64\\" } |
        Sort-Object FullName -Descending | Select-Object -First 1
    if (-not $found) { throw "$Name not found. Install the Windows SDK (it ships with Visual Studio's 'Desktop development with C++' workload)." }
    return $found.FullName
}

$tauriConf = Get-Content (Join-Path $TauriDir "tauri.conf.json") -Raw | ConvertFrom-Json
$Version = $tauriConf.version
$MsixVersion = "$Version.0"

Write-Host "==> Radiocove v$Version ($Arch)" -ForegroundColor Cyan

$ExeSrc = Join-Path $TauriDir "target\release\radiocove.exe"
if (-not (Test-Path $ExeSrc)) { throw "Build output not found at $ExeSrc. Build it first (cargo build --release / npm run tauri build)." }

Write-Host "==> Staging payload" -ForegroundColor Cyan
if (Test-Path $StagingDir) { Remove-Item $StagingDir -Recurse -Force }
# Matches the real Store package layout: the exe lives under the VFS Local AppData
# redirect, at the same path the NSIS installer puts it on a real machine.
$VfsExeDir = Join-Path $StagingDir "VFS\Local AppData\Radiocove"
New-Item -ItemType Directory -Path $VfsExeDir -Force | Out-Null
New-Item -ItemType Directory -Path (Join-Path $StagingDir "Assets") | Out-Null

Copy-Item $ExeSrc (Join-Path $VfsExeDir "radiocove.exe")

$AssetsSrcDir = Join-Path $MsixDir "Assets"
Get-ChildItem $AssetsSrcDir -Filter "*.png" | ForEach-Object {
    Copy-Item $_.FullName (Join-Path $StagingDir "Assets\$($_.Name)")
}

$Template = Get-Content (Join-Path $MsixDir "AppxManifest.template.xml") -Raw
$Manifest = $Template -replace '\{\{VERSION\}\}', $MsixVersion -replace '\{\{ARCH\}\}', $ArchMsix[$Arch]
Set-Content -Path (Join-Path $StagingDir "AppxManifest.xml") -Value $Manifest -Encoding UTF8

# Resources.pri indexes the scale/targetsize-qualified asset files above so Windows can
# pick the right variant for the display's DPI - built fresh from the staged Assets,
# not copied from anywhere.
Write-Host "==> Indexing resources (makepri)" -ForegroundColor Cyan
$makepri = Find-SdkTool "makepri.exe"
& $makepri new /pr $StagingDir /cf (Join-Path $MsixDir "priconfig.xml") /of (Join-Path $StagingDir "Resources.pri") /o | Out-Null
if ($LASTEXITCODE -ne 0) { throw "makepri failed with exit code $LASTEXITCODE" }

New-Item -ItemType Directory -Path $OutDir -Force | Out-Null
$ManifestXml = [xml]$Manifest
$IdentityName = $ManifestXml.Package.Identity.Name
$Publisher = $ManifestXml.Package.Identity.Publisher
$PublisherId = Get-PublisherId $Publisher
$MsixName = "${IdentityName}_${MsixVersion}_$($ArchMsix[$Arch])__${PublisherId}.msix"
$MsixPath = Join-Path $OutDir $MsixName

Write-Host "==> Packing $MsixName" -ForegroundColor Cyan
$makeappx = Find-SdkTool "makeappx.exe"
& $makeappx pack /d $StagingDir /p $MsixPath /overwrite
if ($LASTEXITCODE -ne 0) { throw "makeappx failed with exit code $LASTEXITCODE" }

if ($Sign) {
    New-Item -ItemType Directory -Path $CertDir -Force | Out-Null
    $PfxPath = Join-Path $CertDir "radiocove-test.pfx"
    $CerPath = Join-Path $CertDir "radiocove-test.cer"
    $CertSubject = "CN=AB590003-9108-4489-A869-366AA4C19104"
    $PfxPassword = "radiocove"

    if (-not (Test-Path $PfxPath)) {
        Write-Host "==> Creating local test certificate ($CertSubject)" -ForegroundColor Cyan
        $cert = New-SelfSignedCertificate -Type Custom -Subject $CertSubject -KeyUsage DigitalSignature `
            -FriendlyName "Radiocove MSIX Test Cert" -CertStoreLocation "Cert:\CurrentUser\My" `
            -TextExtension @("2.5.29.37={text}1.3.6.1.5.5.7.3.3", "2.5.29.19={text}false")
        $securePwd = ConvertTo-SecureString -String $PfxPassword -Force -AsPlainText
        Export-PfxCertificate -Cert $cert -FilePath $PfxPath -Password $securePwd | Out-Null
        Export-Certificate -Cert $cert -FilePath $CerPath | Out-Null
    }

    # Self-signed cert is its own root, so Windows needs it in Trusted Root (chain
    # validation), not just Trusted People (publisher allowlist) - both are required
    # for Add-AppxPackage to accept it. Re-checked every run (cheap, idempotent) in
    # case an earlier run created the cert but couldn't get admin to trust it yet.
    $alreadyTrusted = Get-ChildItem "Cert:\LocalMachine\Root" -ErrorAction SilentlyContinue |
        Where-Object { $_.Subject -eq $CertSubject }
    if (-not $alreadyTrusted) {
        Write-Host "==> Trusting certificate for local installs (admin required)" -ForegroundColor Yellow
        try {
            Import-Certificate -FilePath $CerPath -CertStoreLocation "Cert:\LocalMachine\Root" -ErrorAction Stop | Out-Null
            Import-Certificate -FilePath $CerPath -CertStoreLocation "Cert:\LocalMachine\TrustedPeople" -ErrorAction Stop | Out-Null
        } catch {
            Write-Host "    Not running as admin - requesting elevation for this one step (UAC prompt)..." -ForegroundColor Yellow
            $importCmd = "Import-Certificate -FilePath '$CerPath' -CertStoreLocation 'Cert:\LocalMachine\Root'; Import-Certificate -FilePath '$CerPath' -CertStoreLocation 'Cert:\LocalMachine\TrustedPeople'"
            $proc = Start-Process powershell -ArgumentList "-NoProfile", "-Command", $importCmd -Verb RunAs -Wait -PassThru
            if ($proc.ExitCode -ne 0) {
                Write-Host "    Elevation failed or was declined. Manually import $CerPath into 'Trusted Root Certification Authorities' and 'Trusted People' to install locally." -ForegroundColor Red
            }
        }
    }

    Write-Host "==> Signing package" -ForegroundColor Cyan
    $signtool = Find-SdkTool "signtool.exe"
    & $signtool sign /fd SHA256 /f $PfxPath /p $PfxPassword $MsixPath
    if ($LASTEXITCODE -ne 0) { throw "signtool failed with exit code $LASTEXITCODE" }
}

Write-Host "==> Done: $MsixPath" -ForegroundColor Green

if ($Install) {
    if (-not $Sign) {
        Write-Host "==> -Install requires a signed, trusted package. Re-run with -Sign too." -ForegroundColor Red
    } else {
        Write-Host "==> Installing" -ForegroundColor Cyan
        Add-AppxPackage -Path $MsixPath
    }
}
