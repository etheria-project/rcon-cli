@echo off
setlocal enabledelayedexpansion

REM Release helper script for rcon-cli (Windows)
REM Usage: scripts\release.bat <version>
REM Example: scripts\release.bat 1.1.0

if "%~1"=="" (
    echo Usage: %0 ^<version^>
    echo Example: %0 1.1.0
    exit /b 1
)

set VERSION=%1
set TAG=v%VERSION%

echo 🚀 Preparing release %VERSION%

REM Check if working directory is clean
git status --porcelain > temp_status.txt
for %%A in (temp_status.txt) do set size=%%~zA
del temp_status.txt
if !size! GTR 0 (
    echo ❌ Working directory is not clean. Please commit or stash your changes.
    git status --short
    exit /b 1
)

REM Update version in Cargo.toml
echo 📝 Updating version in Cargo.toml
powershell -Command "(Get-Content Cargo.toml) -replace '^version = \".*\"', 'version = \"%VERSION%\"' | Set-Content Cargo.toml"

REM Update Cargo.lock
echo 🔧 Updating Cargo.lock
cargo update -p rcon-cli

REM Check if version exists in changelog
findstr /R "## \[%VERSION%\]" changelog.md >nul
if %errorlevel% equ 0 (
    echo ✅ Version %VERSION% found in changelog
) else (
    echo ⚠️  Version %VERSION% not found in changelog.md
    echo Please update your changelog manually with the new version before continuing.
    echo.
    echo Add a section like this to your changelog.md:
    echo.
    echo ## [%VERSION%] - %date%
    echo.
    echo ### Added
    echo - New features
    echo.
    echo ### Changed
    echo - Changes to existing functionality
    echo.
    echo ### Fixed
    echo - Bug fixes
    echo.
    set /p answer="Have you updated the changelog? (y/N) "
    if /i not "!answer!"=="y" (
        git checkout -- Cargo.toml Cargo.lock
        exit /b 1
    )
)

REM Build and test
echo 🔨 Building project
cargo build --release
if %errorlevel% neq 0 (
    echo ❌ Build failed
    exit /b 1
)

echo 🧪 Running tests
cargo test
if %errorlevel% neq 0 (
    echo ❌ Tests failed
    exit /b 1
)

REM Commit version update
echo 💾 Committing version update
git add Cargo.toml Cargo.lock changelog.md
git commit -m "Bump version to %VERSION%"

REM Create and push tag
echo 🏷️  Creating tag %TAG%
git tag -a "%TAG%" -m "Release %VERSION%"

echo 📤 Pushing changes and tag
git push origin HEAD
git push origin "%TAG%"

echo.
echo ✅ Release %VERSION% has been prepared and pushed!
echo.
echo The GitHub Actions workflow will now:
echo   1. Build binaries for all platforms
echo   2. Create a GitHub release with changelog content
echo   3. Upload release assets
echo.
echo Check the Actions tab in your GitHub repository to monitor the progress.

REM Get repository URL for release link
for /f "tokens=*" %%i in ('git config --get remote.origin.url') do set REPO_URL=%%i
set REPO_URL=%REPO_URL:https://github.com/=%
set REPO_URL=%REPO_URL:.git=%
echo Release URL: https://github.com/%REPO_URL%/releases/tag/%TAG%

endlocal
