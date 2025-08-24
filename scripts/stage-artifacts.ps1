Remove-Item -Recurse -Force artifacts -ErrorAction SilentlyContinue

New-Item -ItemType Directory -Force -Path artifacts | Out-Null

$bins = @(
    'target/debug/lithicrivers-client.exe',
)

# add demo bins using globbing
$bins += Get-ChildItem -Path 'target/debug/demo_*.exe' -Name

foreach ($b in $bins) 
{ 
    if (Test-Path $b) 
    { 
        Copy-Item -Force $b artifacts/ 
    } 
}