@echo off
setlocal
pushd "%~dp0" || exit /b 1

if not exist "websad.exe" goto missing
if not exist "frontend\dist\index.html" goto missing
if not exist "htra_api\websa_san90_sdk_session.dll" goto missing
if not exist "htra_api\htra_api.dll" goto missing

set "WEBSA_HOST=0.0.0.0"
set "WEBSA_PORT=8080"
set "WEBSA_STATIC=%CD%\frontend"
set "WEBSA_RUST_IQS_SHIM=%CD%\htra_api\websa_san90_sdk_session.dll"

echo WebSA  -  HAROGIC hardware mode
echo On this PC: http://127.0.0.1:8080/
echo From another PC: use this PC's LAN IP address and port 8080.
echo Press Ctrl+C to stop the service.
echo.
".\websad.exe"
set "RESULT=%ERRORLEVEL%"
if not "%RESULT%"=="0" (
    echo.
    echo WebSA exited with code %RESULT%.
    pause
)
popd
exit /b %RESULT%

:missing
echo WebSA files are missing. Extract the entire ZIP and try again.
pause
popd
exit /b 1
