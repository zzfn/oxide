@echo off
REM Oxide CLI - npm wrapper script for Windows

setlocal

set BIN_DIR=%~dp0
set BINARY=%BIN_DIR%..\lib\oxide.exe

REM 如果 lib/oxide.exe 不存在，尝试使用 bin 目录下的二进制
if not exist "%BINARY%" (
    set BINARY=%BIN_DIR%oxide.exe
)

REM 执行二进制文件
"%BINARY%" %*
