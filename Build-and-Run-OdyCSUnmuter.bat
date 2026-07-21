@echo off
setlocal EnableExtensions EnableDelayedExpansion
cd /d "%~dp0"

title OdyCSUnmuter Beta - Source Build

where cargo >nul 2>nul
if errorlevel 1 (
    echo Rust/Cargo was not found.
    echo.
    echo Source builders must install Rust through rustup.
    echo Windows may also require Microsoft C++ Build Tools.
    echo.
    echo Ordinary users should download the compiled Windows release;
    echo the compiled release does NOT require Rust.
    pause
    exit /b 1
)

if "%~1"=="" (
    echo Drag an UNTOUCHED ORIGINAL .dem file onto:
    echo   Build-and-Run-OdyCSUnmuter.bat
    echo.
    echo Or run:
    echo   Build-and-Run-OdyCSUnmuter.bat "C:\path\match.dem"
    pause
    exit /b 1
)

if not exist "vendor\source2-demo\Cargo.toml" (
    echo Preparing the local parser dependency...
    powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0prepare-faceit-parser.ps1"
    set "PREP_EXIT=!ERRORLEVEL!"

    if not "!PREP_EXIT!"=="0" (
        echo.
        echo Parser preparation failed with exit code !PREP_EXIT!.
        pause
        exit /b !PREP_EXIT!
    )

    echo Parser preparation completed successfully.
)

echo.
echo Building OdyCSUnmuter Beta...
cargo build --release
set "BUILD_EXIT=!ERRORLEVEL!"

if not "!BUILD_EXIT!"=="0" (
    echo.
    echo Build failed with exit code !BUILD_EXIT!.
    pause
    exit /b !BUILD_EXIT!
)

echo.
echo Starting rewrite...
"%~dp0target\release\odycs-unmuter.exe" "%~1"
set "RUN_EXIT=!ERRORLEVEL!"

if not "!RUN_EXIT!"=="0" (
    echo.
    echo Rewrite FAILED with exit code !RUN_EXIT!.
    echo Do not use any .partial file.
    pause
    exit /b !RUN_EXIT!
)

echo.
echo Rewrite completed successfully.
echo Test only the newly created *_OdyCSunmuted.dem file.
pause
