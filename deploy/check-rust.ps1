#Requires -Version 7.3
# Проверка Rust в локальном Docker без запуска игровых служб.
[CmdletBinding()]
param(
    [ValidateSet('Check', 'Build')]
    [string]$Mode = 'Check',
    [ValidateRange(128, 16384)]
    [int]$CacheLimitMiB = 2048
)

$ErrorActionPreference = 'Stop'
$rustDirectory = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '../server/rust')).Path
$cacheVolume = 'nebokrai-rust-check-cache'
$image = 'rust:1.97.1-bookworm'

# Один выделенный том: зависимости и target. Исходники доступны только для чтения.
# Очищаются исключительно фиксированные пути внутри этого тома.
$checkScript = @'
set -eu
cache=/nebokrai-cache
mkdir -p "$cache"
clear_cache() {
    rm -rf /nebokrai-cache/target /nebokrai-cache/cargo
}
trim_cache() {
    size=$(du -sm "$cache" | cut -f1)
    if [ "$size" -gt "$NEBOKRAI_CACHE_LIMIT_MIB" ]; then
        echo "Cache limit exceeded ($size MiB): removing Nebokrai target artifacts"
        rm -rf /nebokrai-cache/target
        size=$(du -sm "$cache" | cut -f1)
        if [ "$size" -gt "$NEBOKRAI_CACHE_LIMIT_MIB" ]; then
            clear_cache
        fi
    fi
    echo "Nebokrai cache: $(du -sm "$cache" | cut -f1) MiB"
}
key=$( { rustc --version; sha256sum Cargo.lock; printf '%s\n' 'debug=0 incremental=0'; } | sha256sum | cut -d' ' -f1)
if [ -f "$cache/key" ] && [ "$(cat "$cache/key")" != "$key" ]; then
    clear_cache
fi
printf '%s\n' "$key" > "$cache/key"
trim_cache
trap 'status=$?; trap - EXIT; trim_cache; exit "$status"' EXIT
cargo "$@"
if [ "$1" = build ]; then
    for binary in authserver loginserver worldserver gameserver billingserver miscserver; do
        install "$cache/target/debug/$binary" "/artifacts/.$binary.new"
        mv -f "/artifacts/.$binary.new" "/artifacts/$binary"
    done
fi
'@
$checkScript = $checkScript.Replace("`r`n", "`n")

$cargoArguments = @('check', '--locked', '--workspace', '--lib', '--bins')
$artifactMount = @()
if ($Mode -eq 'Build') {
    $artifactDirectory = Join-Path $PSScriptRoot '../.local/rust-bin'
    New-Item -ItemType Directory -Path $artifactDirectory -Force | Out-Null
    $artifactDirectory = (Resolve-Path -LiteralPath $artifactDirectory).Path
    $artifactMount = @('--mount', "type=bind,source=$artifactDirectory,target=/artifacts")
    $cargoArguments = @('build', '--locked', '--bins')
}

$dockerArguments = @(
    '--context', 'desktop-linux', 'run', '--rm',
    '--name', 'nebokrai-rust-check',
    '--label', 'nebokrai.purpose=rust-check',
    '--mount', "type=bind,source=$rustDirectory,target=/source,readonly",
    '--mount', "type=volume,source=$cacheVolume,target=/nebokrai-cache",
    '--workdir', '/source',
    '--env', 'CARGO_HOME=/nebokrai-cache/cargo',
    '--env', 'CARGO_TARGET_DIR=/nebokrai-cache/target',
    '--env', 'CARGO_INCREMENTAL=0',
    '--env', 'RUSTUP_TOOLCHAIN=1.97.1',
    '--env', 'CARGO_BUILD_JOBS=2',
    '--env', 'CARGO_PROFILE_DEV_DEBUG=0',
    '--env', "NEBOKRAI_CACHE_LIMIT_MIB=$CacheLimitMiB",
    $artifactMount
    $image, 'sh', '-c', $checkScript, 'nebokrai-check'
)
& docker @dockerArguments @cargoArguments
exit $LASTEXITCODE
