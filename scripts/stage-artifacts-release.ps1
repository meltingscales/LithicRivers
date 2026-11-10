Remove-Item -Recurse -Force "./artifacts/" -ErrorAction SilentlyContinue

New-Item -ItemType Directory -Force -Path "./artifacts/" | Out-Null

$bins = @(
    'target/release/lithicrivers-client.exe'
)

# add demo bins using globbing
$demo_bins = @()
$demo_bins += Get-ChildItem -Path 'target/release/demo_*.exe' -Name
$demo_bins += Get-ChildItem -Path 'target/release/utility_*.exe' -Name

# append demo bins to $bins
foreach ($b in $demo_bins)
{
    $bins += "target/release/" + $b
}

write-output $bins

foreach ($b in $bins)
{
    if (Test-Path $b)
    {
        Copy-Item -Force $b "./artifacts/"
    }
}

# add the Steam API library the release binary loads, from where the steamworks-sys build left it
$steam_lib = Get-ChildItem -Path 'target/release/build' -Recurse -Filter 'steam_api64.dll' |
    Where-Object { $_.FullName -like '*steamworks-sys*' } | Select-Object -First 1
if (-not $steam_lib)
{
    Write-Error "Steam API library not found; was the release built with --features steam?"
    exit 1
}
Copy-Item -Force $steam_lib.FullName "./artifacts/"

# add changelog
Copy-Item -Force CHANGELOG.txt "./artifacts/"

# add license and third party notices
Copy-Item -Force LICENSE.txt "./artifacts/"
Copy-Item -Force THIRD-PARTY-NOTICES.txt "./artifacts/"

# add launcher script
Copy-Item -Force scripts/launcher/lithicrivers-launcher.bat "./artifacts/"