try {
    Remove-Item -Recurse -Force artifacts -ErrorAction SilentlyContinue
} catch {
    Write-Output "Failed to remove artifacts folder. Continuing..."
}