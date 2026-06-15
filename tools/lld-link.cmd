@echo off
if not defined RUSTUP_HOME set "RUSTUP_HOME=%USERPROFILE%\.rustup"
set "TOOLCHAIN=%RUSTUP_HOME%\toolchains\stable-x86_64-pc-windows-gnu"
set "LLD=%TOOLCHAIN%\lib\rustlib\x86_64-pc-windows-gnu\bin\rust-lld.exe"
set "SYSLIB=%TOOLCHAIN%\lib\rustlib\x86_64-pc-windows-gnu\lib\self-contained"

echo %* | findstr /C:"-shared" >nul 2>&1
if %ERRORLEVEL%==0 (
    set "CRT=%SYSLIB%\dllcrt2.o"
) else (
    set "CRT=%SYSLIB%\crt2.o"
)

"%LLD%" %* "%CRT%" "-L%SYSLIB%"
