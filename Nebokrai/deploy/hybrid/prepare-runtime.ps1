param(
    [string]$ClientAddress = "127.0.0.1"
)

$ErrorActionPreference = "Stop"
$runtime = (Resolve-Path (Join-Path $PSScriptRoot "..\..\runtime")).Path
$encoding = [System.Text.Encoding]::GetEncoding(1251)

function Update-Value {
    param(
        [string]$Path,
        [string]$Key,
        [string]$Value
    )

    $text = [System.IO.File]::ReadAllText($Path, $encoding)
    $pattern = "(?m)^(\s*" + [regex]::Escape($Key) + "\s+).*$"
    if (-not [regex]::IsMatch($text, $pattern)) {
        throw "В $Path отсутствует параметр $Key"
    }
    $text = [regex]::Replace($text, $pattern, { param($match) $match.Groups[1].Value + $Value }, 1)
    [System.IO.File]::WriteAllText($Path, $text, $encoding)
}

$auth = Join-Path $runtime "AuthServer\setup.ini"
Update-Value $auth "DatabaseIP" "mssql"
Update-Value $auth "LogDatabaseIP" "mssql"

$billing = Join-Path $runtime "BillingServer\setup.ini"
Update-Value $billing "SqlServerIP" "mssql"
Update-Value $billing "LogSvrIP" "mssql"

$login = Join-Path $runtime "LoginServer\setup.ini"
Update-Value $login "SqlServerIP" "mssql"
Update-Value $login "ServerInfoLogIP" "mssql"
Update-Value $login "BillingDatabaseIP" "mssql"
[System.IO.File]::WriteAllText(
    (Join-Path $runtime "LoginServer\aslist.ini"),
    "miracle_auth`t7100`r`n",
    $encoding
)

$misc = Join-Path $runtime "MiscServer\setup.ini"
Update-Value $misc "WorldIP" "miracle_world"
Update-Value $misc "LocalIP" "miracle_misc"

$world = Join-Path $runtime "WorldServer\setup.ini"
Update-Value $world "LoginIP" "miracle_login"
Update-Value $world "SqlServerIP" "mssql"
Update-Value $world "LogSysServer" "mssql"
Update-Value $world "CostDBIP" "mssql"
Update-Value $world "CostDBOnLoginIP" "mssql"

$game = Join-Path $runtime "GameServer\setup.ini"
Update-Value $game "LoginIP" "miracle_world"
Update-Value $game "BillingIP" "miracle_billing"
Update-Value $game "BillingIPBak" "miracle_billing"
Update-Value $game "BindPortForBS" "1993"
Update-Value $game "LocalIP" $ClientAddress

$serverSetup = Join-Path $runtime "WorldServer\serversetup.ini"
$serverText = [System.IO.File]::ReadAllText($serverSetup, $encoding)
$serverText = [regex]::Replace(
    $serverText,
    "(?m)^#\s*1\s+\S+\s+2347\s*$",
    "#`t1`t$ClientAddress`t2347"
)
[System.IO.File]::WriteAllText($serverSetup, $serverText, $encoding)

$billingSetup = Join-Path $runtime "BillingServer\gsinfosetup.ini"
$billingText = [System.IO.File]::ReadAllText($billingSetup, $encoding)
if ($billingText -notmatch "(?m)^\s*BindClient\s+[01]\s*$") {
    $billingText = "BindClient`t1`r`n" + $billingText
}
$billingText = [regex]::Replace(
    $billingText,
    "(?m)^#\s*\S+\s+1993\s*$",
    "#`tmiracle_game`t1993"
)
[System.IO.File]::WriteAllText($billingSetup, $billingText, $encoding)

$passwordMatch = [regex]::Match(
    [System.IO.File]::ReadAllText($auth, $encoding),
    "(?m)^DatabasePassword\s+(\S+)"
)
if (-not $passwordMatch.Success) {
    throw "Не найден пароль Miracle в AuthServer/setup.ini"
}

$environmentPath = Join-Path $PSScriptRoot ".env"
if (-not (Test-Path -LiteralPath $environmentPath)) {
    $saPassword = "Nebokrai!" + [guid]::NewGuid().ToString("N")
    $environment = @(
        "MSSQL_SA_PASSWORD=$saPassword"
        "MIRACLE_DB_PASSWORD=$($passwordMatch.Groups[1].Value)"
    ) -join "`n"
    [System.IO.File]::WriteAllText($environmentPath, $environment + "`n", [System.Text.Encoding]::UTF8)
}

Write-Output "Runtime подготовлен для клиента $ClientAddress"
