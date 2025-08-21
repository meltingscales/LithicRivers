Remove-Item -Recurse -Force artifacts -ErrorAction SilentlyContinue

New-Item -ItemType Directory -Force -Path artifacts | Out-Null

$bins = @(
    'target/debug/lithicrivers-client.exe',
    'target/debug/demo_inventory.exe',
    'target/debug/demo_body.exe',
    'target/debug/beezzaroll_color_test.exe',
    'target/debug/beezzaroll_sprite_test.exe',
    'target/debug/portrait_sprite_test.exe'
)
foreach ($b in $bins) 
{ 
    if (Test-Path $b) 
    { 
        Copy-Item -Force $b artifacts/ 
    } 
}