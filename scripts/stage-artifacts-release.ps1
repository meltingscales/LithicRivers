Remove-Item -Recurse -Force artifacts -ErrorAction SilentlyContinue

New-Item -ItemType Directory -Force -Path artifacts | Out-Null

$bins = @(
    'target/release/lithicrivers-client.exe',
    'target/release/demo_inventory.exe',
    'target/release/demo_body.exe',
    'target/release/beezzaroll_color_test.exe',
    'target/release/beezzaroll_sprite_test.exe',
    'target/release/portrait_sprite_test.exe'
)
foreach ($b in $bins) 
{ 
    if (Test-Path $b) 
    { 
        Copy-Item -Force $b artifacts/ 
    } 
}
