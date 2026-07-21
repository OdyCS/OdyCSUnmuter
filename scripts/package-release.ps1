param(
    [Parameter(Mandatory = $true)]
    [string]$Version
)

$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
$dist = Join-Path $root "dist"
$packageName = "OdyCSUnmuter-Beta-$Version-windows-x64"
$packageDir = Join-Path $dist $packageName
$zipPath = Join-Path $dist "$packageName.zip"
$zipHashPath = "$zipPath.sha256"
$exeSource = Join-Path $root "target\release\odycs-unmuter.exe"

if (-not (Test-Path $exeSource)) {
    throw "Compiled executable not found: $exeSource. Run cargo build --release first."
}

if (Test-Path $packageDir) {
    Remove-Item $packageDir -Recurse -Force
}
if (Test-Path $zipPath) {
    Remove-Item $zipPath -Force
}
if (Test-Path $zipHashPath) {
    Remove-Item $zipHashPath -Force
}

New-Item -ItemType Directory -Force -Path $packageDir | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $packageDir "THIRD_PARTY_LICENSES") | Out-Null

Copy-Item $exeSource (Join-Path $packageDir "OdyCSUnmuter.exe")
Copy-Item (Join-Path $root "release\Run-OdyCSUnmuter.bat") $packageDir
Copy-Item (Join-Path $root "release\README.txt") $packageDir
Copy-Item (Join-Path $root "release\CS2-CONSOLE-COMMANDS.txt") $packageDir
Copy-Item (Join-Path $root "LICENSE") $packageDir
Copy-Item (Join-Path $root "THIRD_PARTY_NOTICES.md") $packageDir

$thirdPartyLicense = Join-Path $root "vendor\source2-demo\LICENSE-MIT"
if (Test-Path $thirdPartyLicense) {
    Copy-Item $thirdPartyLicense (Join-Path $packageDir "THIRD_PARTY_LICENSES\source2-demo-LICENSE-MIT")
}

$manifestLines = foreach ($file in Get-ChildItem $packageDir -File -Recurse | Sort-Object FullName) {
    $relative = $file.FullName.Substring($packageDir.Length + 1).Replace('\', '/')
    $hash = (Get-FileHash $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
    "$hash  $relative"
}
$manifestLines | Set-Content -Path (Join-Path $packageDir "SHA256SUMS.txt") -Encoding ascii

Compress-Archive -Path (Join-Path $packageDir "*") -DestinationPath $zipPath -Force
$zipHash = (Get-FileHash $zipPath -Algorithm SHA256).Hash.ToLowerInvariant()
"$zipHash  $([IO.Path]::GetFileName($zipPath))" | Set-Content -Path $zipHashPath -Encoding ascii

Write-Host "Created release ZIP:"
Write-Host "  $zipPath"
Write-Host "Created SHA-256 file:"
Write-Host "  $zipHashPath"
