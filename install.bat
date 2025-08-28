@echo off
echo Testing new statusline...
echo.

REM 备份旧文件并替换新文件
if exist "C:\Users\zhang\.claude\ccline\statusline.exe" (
    move /Y "C:\Users\zhang\.claude\ccline\statusline.exe" "C:\Users\zhang\.claude\ccline\statusline.exe.old" >nul 2>&1
)
move /Y "C:\Users\zhang\.claude\ccline\statusline_new.exe" "C:\Users\zhang\.claude\ccline\statusline.exe"

echo New statusline has been installed successfully!
echo.
echo Please restart Claude Code to use the new version.
echo.
echo Changes in this version:
echo - Fixed API endpoint to use /api/backend/users/info
echo - Updated data structure to match packycode-cost project
echo - Removed node speed test feature
echo - Removed Opus status indicator
echo - Improved quota display format (shows remaining balance and percentage)
echo.
pause