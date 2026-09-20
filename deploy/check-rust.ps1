# Проверка Rust в локальном Docker без запуска игровых служб.
[CmdletBinding()]
param(
    [ValidateSet('Check', 'Test')]
    [string]$Mode = 'Check',
    [string]$TestFilter = '',
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
'@

$cargoArguments = if ($Mode -eq 'Check') {
    @('check', '--locked', '--all-targets')
} else {
    @('test', '--locked', '--lib')
}
if ($TestFilter) {
    if ($Mode -ne 'Test') { throw 'TestFilter применим только к режиму Test' }
    $cargoArguments += $TestFilter
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
    '--env', 'CARGO_PROFILE_TEST_DEBUG=0',
    '--env', "NEBOKRAI_CACHE_LIMIT_MIB=$CacheLimitMiB",
    $image, 'sh', '-c', $checkScript, 'nebokrai-check'
)
& docker @dockerArguments @cargoArguments
exit $LASTEXITCODE
