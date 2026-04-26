#Requires -Version 5.1

[CmdletBinding()]
param(
    [Parameter(Mandatory=$true, Position=0)]
    [string]$Version
)

# Validate version format (basic check for vX.Y.Z format)
if ($Version -notmatch '^v[0-9]+\.[0-9]+\.[0-9]+.*$') {
    Write-Host "❌ Error: Version must be in format 'vX.Y.Z' (e.g. v1.0.0)" -ForegroundColor Red
    exit 1
}

# Check if tag already exists
 $existingTags = git tag -l
if ($existingTags -match "^$Version$") {
    Write-Host "❌ Error: Tag '$Version' already exists" -ForegroundColor Red
    git tag -l | Select-String "$Version"
    exit 1
}

# Check if working directory is clean
 $statusOutput = git status --porcelain
if ($statusOutput) {
    Write-Host "⚠️  Working directory has uncommitted changes:" -ForegroundColor Yellow
    git status --short
    Write-Host ""
    $reply = Read-Host "Continue anyway? [y/N]"
    if ($reply -notmatch '^[Yy]$') {
        Write-Host "❌ Aborted" -ForegroundColor Red
        exit 1
    }
}

# Show changelog for review
Write-Host "📄 Current CHANGELOG.txt (first 10 lines):"
Write-Host "----------------------------------------"
if (Test-Path "CHANGELOG.txt") {
    Get-Content "CHANGELOG.txt" -TotalCount 10
} else {
    Write-Host "⚠️  CHANGELOG.txt not found" -ForegroundColor Yellow
}
Write-Host "----------------------------------------"
Write-Host ""
 $reply = Read-Host "Have you reviewed and updated the CHANGELOG.txt for this release '$Version'? [y/N]"
if ($reply -notmatch '^[Yy]$') {
    Write-Host "❌ Please update CHANGELOG.txt before releasing" -ForegroundColor Red
    exit 1
}

# Final confirmation before publishing
Write-Host ""
Write-Host "🚀 Ready to publish release $Version"
Write-Host "📋 This will:"
Write-Host "   - Create commit and tag $Version "
Write-Host "   - Push to GitHub (triggering CI/CD)"
Write-Host "   - Build for Windows, macOS, and Linux"
Write-Host "   - Upload to GitHub Releases"
Write-Host "   - Upload to Steam via SteamPipe"
Write-Host ""
 $reply = Read-Host "Are you sure you want to trigger a build and upload $Version to GitHub and Steam? [y/N]"
if ($reply -notmatch '^[Yy]$') {
    Write-Host "❌ Aborted" -ForegroundColor Red
    exit 1
}

Write-Host "🚀 Publishing release $Version..."

# Add all changes
git add .

# Create commit with version
try {
    git commit -m "Release $Version"
} catch {
    Write-Host "⚠️  No changes to commit" -ForegroundColor Yellow
}

# Create and push tag
git tag "$Version"

Write-Host "📤 Pushing to remote..."
git push origin
git push origin --tags

Write-Host "✅ Successfully published release $Version" -ForegroundColor Green

# Extract GitHub repo from remote URL
$remoteUrl = git remote get-url origin
$repoPath = $remoteUrl -replace '.*github.com[\/:]', '' -replace '\.git$',''
Write-Host "  GitHub Actions will now build and deploy the release"
Write-Host "  Monitor the release at: https://github.com/$repoPath/releases"
