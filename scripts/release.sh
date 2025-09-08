#!/bin/bash

# Release helper script for rcon-cli
# Usage: ./scripts/release.sh <version>
# Example: ./scripts/release.sh 1.1.0

set -e

if [ $# -ne 1 ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 1.1.0"
    exit 1
fi

VERSION="$1"
TAG="v$VERSION"
DATE=$(date +%Y-%m-%d)

echo "🚀 Preparing release $VERSION"

# Check if we're on the main branch or rust-cli branch
CURRENT_BRANCH=$(git branch --show-current)
if [ "$CURRENT_BRANCH" != "main" ] && [ "$CURRENT_BRANCH" != "rust-cli" ]; then
    echo "⚠️  Warning: You're not on main or rust-cli branch. Current branch: $CURRENT_BRANCH"
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Check if working directory is clean
if [ -n "$(git status --porcelain)" ]; then
    echo "❌ Working directory is not clean. Please commit or stash your changes."
    git status --short
    exit 1
fi

# Update version in Cargo.toml
echo "📝 Updating version in Cargo.toml"
sed -i.bak "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
rm Cargo.toml.bak 2>/dev/null || true

# Update Cargo.lock
echo "🔧 Updating Cargo.lock"
cargo update -p rcon-cli

# Check if version exists in changelog
if grep -q "## \[$VERSION\]" changelog.md; then
    echo "✅ Version $VERSION found in changelog"
else
    echo "⚠️  Version $VERSION not found in changelog.md"
    echo "Please update your changelog manually with the new version before continuing."
    echo ""
    echo "Add a section like this to your changelog.md:"
    echo ""
    echo "## [$VERSION] - $DATE"
    echo ""
    echo "### Added"
    echo "- New features"
    echo ""
    echo "### Changed"
    echo "- Changes to existing functionality"
    echo ""
    echo "### Fixed"
    echo "- Bug fixes"
    echo ""
    read -p "Have you updated the changelog? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        # Restore Cargo.toml
        git checkout -- Cargo.toml Cargo.lock
        exit 1
    fi
fi

# Build and test
echo "🔨 Building project"
cargo build --release

echo "🧪 Running tests"
cargo test

# Commit version update
echo "💾 Committing version update"
git add Cargo.toml Cargo.lock changelog.md
git commit -m "Bump version to $VERSION"

# Create and push tag
echo "🏷️  Creating tag $TAG"
git tag -a "$TAG" -m "Release $VERSION"

echo "📤 Pushing changes and tag"
git push origin HEAD
git push origin "$TAG"

echo ""
echo "✅ Release $VERSION has been prepared and pushed!"
echo ""
echo "The GitHub Actions workflow will now:"
echo "  1. Build binaries for all platforms"
echo "  2. Create a GitHub release with changelog content"
echo "  3. Upload release assets"
echo ""
echo "Check the Actions tab in your GitHub repository to monitor the progress."
echo "Release URL: https://github.com/$(git config --get remote.origin.url | sed 's/.*github.com[:/]\(.*\)\.git/\1/')/releases/tag/$TAG"
