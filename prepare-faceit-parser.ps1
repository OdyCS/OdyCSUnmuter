$ErrorActionPreference = "Stop"

$project = Split-Path -Parent $MyInvocation.MyCommand.Path
$vendorRoot = Join-Path $project "vendor"
$destination = Join-Path $vendorRoot "source2-demo"
$destinationManifest = Join-Path $destination "Cargo.toml"
$parserFile = Join-Path $destination "src\parser\mod.rs"

if ((Test-Path $destinationManifest) -and (Test-Path $parserFile)) {
    $existing = [System.IO.File]::ReadAllText($parserFile)
    if ($existing.Contains("FACEIT_MISSING_FOOTER_PATCH")) {
        Write-Host "FACEIT parser dependency is already prepared."
        exit 0
    }

    Remove-Item $destination -Recurse -Force
}
elseif (Test-Path $destination) {
    Remove-Item $destination -Recurse -Force
}

$bootstrap = Join-Path $project ".source2-demo-bootstrap"
$bootstrapSrc = Join-Path $bootstrap "src"
New-Item -ItemType Directory -Force -Path $bootstrapSrc | Out-Null

$bootstrapToml = @'
[package]
name = "source2-demo-bootstrap"
version = "0.0.0"
edition = "2021"

[dependencies]
source2-demo = { version = "=0.5.8", default-features = false, features = ["cs2"] }
'@

[System.IO.File]::WriteAllText(
    (Join-Path $bootstrap "Cargo.toml"),
    $bootstrapToml,
    [System.Text.UTF8Encoding]::new($false)
)
[System.IO.File]::WriteAllText(
    (Join-Path $bootstrapSrc "main.rs"),
    "fn main() {}",
    [System.Text.UTF8Encoding]::new($false)
)

Write-Host "Downloading source2-demo 0.5.8 through a bootstrap manifest..."
& cargo fetch --manifest-path (Join-Path $bootstrap "Cargo.toml")
if ($LASTEXITCODE -ne 0) {
    throw "Bootstrap cargo fetch failed with exit code $LASTEXITCODE."
}

$cargoHome = $env:CARGO_HOME
if ([string]::IsNullOrWhiteSpace($cargoHome)) {
    $cargoHome = Join-Path $env:USERPROFILE ".cargo"
}

$registrySource = Join-Path $cargoHome "registry\src"
$crate = Get-ChildItem `
    -Path $registrySource `
    -Directory `
    -Recurse `
    -Filter "source2-demo-0.5.8" `
    -ErrorAction SilentlyContinue |
    Select-Object -First 1

if ($null -eq $crate) {
    throw "source2-demo-0.5.8 could not be located under $registrySource"
}

New-Item -ItemType Directory -Force -Path $vendorRoot | Out-Null
Copy-Item -Path $crate.FullName -Destination $destination -Recurse -Force

$checksum = Join-Path $destination ".cargo-checksum.json"
if (Test-Path $checksum) {
    Remove-Item $checksum -Force
}

if (-not (Test-Path $parserFile)) {
    throw "Expected parser source not found: $parserFile"
}

$content = [System.IO.File]::ReadAllText($parserFile)
$pattern = 'let replay_info\s*=\s*Self::read_file_info_from_reader\(&mut reader\)\?;'
$matches = [regex]::Matches($content, $pattern)

if ($matches.Count -ne 1) {
    throw "Expected one strict file-info read; found $($matches.Count)."
}

$replacement = @'
let replay_info = Self::read_file_info_from_reader(&mut reader)
            .unwrap_or_default(); // FACEIT_MISSING_FOOTER_PATCH
'@

$content = [regex]::Replace($content, $pattern, $replacement.TrimEnd(), 1)

[System.IO.File]::WriteAllText(
    $parserFile,
    $content,
    [System.Text.UTF8Encoding]::new($false)
)

Write-Host "Prepared local parser dependency:"
Write-Host "  $parserFile"

exit 0
