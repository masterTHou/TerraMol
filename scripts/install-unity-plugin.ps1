$ErrorActionPreference = 'Stop'
cargo build --release
if ($LASTEXITCODE -ne 0) { throw 'Rust build failed' }
New-Item -ItemType Directory -Force unity/Assets/Plugins/x86_64 | Out-Null
Copy-Item -LiteralPath target/release/mol_engine_core.dll -Destination unity/Assets/Plugins/x86_64/mol_engine_core.dll -Force
