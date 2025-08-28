Remove-Item -Recurse -Force artifacts -ErrorAction SilentlyContinue

New-Item -ItemType Directory -Force -Path artifacts | Out-Null

$bins = @(
    'target/release/lithicrivers-client.exe'
)

# add demo bins using globbing
$demo_bins = @()
$demo_bins += Get-ChildItem -Path 'target/release/demo_*.exe' -Name

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
        Copy-Item -Force $b artifacts/ 
    } 
}
