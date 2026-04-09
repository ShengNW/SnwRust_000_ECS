param(
  [string]$ProjectDir = "D:\exe\SNW_ECS",
  [string]$CargoHome = "D:\exe\SNW_\cargo",
  [string]$RustupHome = "D:\exe\SNW_\rustup",
  [string]$Toolchain = "stable-x86_64-pc-windows-msvc"
)

$ErrorActionPreference = "Stop"
$env:CARGO_HOME = $CargoHome
$env:RUSTUP_HOME = $RustupHome
Set-Location $ProjectDir

cargo +$Toolchain run --release
