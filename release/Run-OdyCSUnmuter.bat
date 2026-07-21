@echo off
setlocal EnableExtensions EnableDelayedExpansion
cd /d "%~dp0"

title OdyCSUnmuter Beta

if "%~1"=="" (
    echo OdyCSUnmuter Beta
    echo.
    echo Drag an UNTOUCHED ORIGINAL CS2 .dem file onto:
    echo   Run-OdyCSUnmuter.bat
    echo.
    echo Or run:
    echo   Run-OdyCSUnmuter.bat "C:\path\match.dem"
    pause
    exit /b 1
)

if not exist "%~dp0OdyCSUnmuter.exe" (
    echo OdyCSUnmuter.exe is missing from this folder.
    echo Extract the COMPLETE release ZIP and try again.
    pause
    exit /b 1
)

"%~dp0OdyCSUnmuter.exe" "%~1"
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
echo.
echo Test only the newly created file ending in:
echo   _OdyCSunmuted.dem
echo.
echo Recommended CS2 console commands:
echo   tv_listen_voice_indices -1
echo   tv_listen_voice_indices_h -1
echo.
echo See CS2-CONSOLE-COMMANDS.txt for explanations.
pause
