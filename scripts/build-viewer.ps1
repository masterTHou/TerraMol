$ErrorActionPreference = 'Stop'
$previousFlags = $env:RUSTFLAGS
try {
    # Static CRT keeps the portable EXE independent of VC++ redistributable installs.
    $env:RUSTFLAGS = "$previousFlags -C target-feature=+crt-static".Trim()
    cargo build --release --features viewer --bin terramol_viewer --target-dir target/portable
    if ($LASTEXITCODE -ne 0) { throw 'Portable viewer build failed' }
    Copy-Item -LiteralPath target/portable/release/terramol_viewer.exe -Destination TerraMol.exe -Force
} finally {
    $env:RUSTFLAGS = $previousFlags
}
