#!/usr/bin/env bash
# Util: Publish Release

set -euo pipefail

version=$1

# Validate version format (basic check for vX.Y.Z format)
if [[ ! "$version" =~ ^v[0-9]+\.[0-9]+\.[0-9]+.*$ ]]; then
    echo "❌ Error: Version must be in format 'vX.Y.Z' (e.g. v1.0.0)"
    exit 1
fi

# Check if tag already exists
if git tag -l | grep -q "^version$"; then
    echo "❌ Error: Tag '$version' already exists"
    git tag -l | grep "$version"
    exit 1
fi

# Check if working directory is clean
if [[ -n $(git status --porcelain) ]]; then
    echo "⚠️  Working directory has uncommitted changes:"
    git status --short
    echo ""
    read -p "Continue anyway? [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "❌ Aborted"
        exit 1
    fi
fi

# Show changelog for review
echo "📄 Current CHANGELOG.txt (first 10 lines):"
echo "----------------------------------------"
head CHANGELOG.txt || echo "⚠️  CHANGELOG.txt not found"
echo "----------------------------------------"
echo ""
read -p "Have you reviewed and updated the CHANGELOG.txt for this release '$version'? [y/N] " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "❌ Please update CHANGELOG.txt before releasing"
    exit 1
fi

# Final confirmation before publishing
echo ""
echo "🚀 Ready to publish release $version"
echo "📋 This will:"
echo "   • Create commit and tag $version"
echo "   • Push to GitHub (triggering CI/CD)"
echo "   • Build for Windows, macOS, and Linux"
echo "   • Upload to GitHub Releases"
echo "   • Upload to Steam via SteamPipe"
echo ""
read -p "Are you sure you want to trigger a build and upload $version to GitHub and Steam? [y/N] " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "❌ Aborted"
    exit 1
fi

echo "🚀 Publishing release $version..."

# Add all changes
git add .

# Create commit with version
git commit -m "Release $version" || echo "⚠️  No changes to commit"

# Create and push tag
git tag "$version"

echo "📤 Pushing to remote..."
git push origin
git push origin --tags

echo "✅ Successfully published release $version"
echo "🔗 GitHub Actions will now build and deploy the release"
echo "🎯 Monitor the release at: https://github.com/$(git remote get-url origin | sed 's/.*github.com[\/:]//;s/.git$//')/releases"
