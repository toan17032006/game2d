@echo off
rem Run a command inside the Visual Studio x64 toolchain environment.
rem Usage: vs-env.cmd <command...>
call "C:\Program Files\Microsoft Visual Studio\18\Community\VC\Auxiliary\Build\vcvars64.bat" >nul 2>&1
if errorlevel 1 (
  echo vs-env: vcvars64.bat failed
  exit /b 1
)
%*
exit /b %errorlevel%
