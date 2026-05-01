@echo off
setlocal enabledelayedexpansion

echo === SD WebUI Manager Build ===
echo.

where rustc >nul 2>nul || (echo Error: rustc not found. Install from https://rustup.rs & exit /b 1)
where node >nul 2>nul || (echo Error: node not found. Install from https://nodejs.org & exit /b 1)
where cargo >nul 2>nul || (echo Error: cargo not found. Install from https://rustup.rs & exit /b 1)

echo [1/3] Installing npm dependencies...
call npm install

echo [2/3] Building Tauri application...
call npm run build

echo [3/3] Done!
echo.
echo Build artifacts located in: src-tauri\target\release\bundle\
