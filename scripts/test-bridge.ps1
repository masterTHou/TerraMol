$ErrorActionPreference = 'Stop'
cargo build --release
if ($LASTEXITCODE -ne 0) { throw 'Rust build failed' }
New-Item -ItemType Directory -Force artifacts | Out-Null
$compiler = "$env:WINDIR\Microsoft.NET\Framework64\v4.0.30319\csc.exe"
& $compiler /nologo /out:artifacts/BridgeSmoke.exe (Resolve-Path unity/Assets/TerraMol/Scripts/RustCoreBridge.cs).Path (Resolve-Path tests/BridgeSmoke.cs).Path
if ($LASTEXITCODE -ne 0) { throw 'C# build failed' }
Copy-Item -LiteralPath target/release/mol_engine_core.dll -Destination artifacts/mol_engine_core.dll -Force
& ./artifacts/BridgeSmoke.exe
if ($LASTEXITCODE -ne 0) { throw 'C# ABI smoke failed' }
