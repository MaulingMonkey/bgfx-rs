:: Copyright (c) 2019, Michael Rickert
:: License: http://opensource.org/licenses/ISC

@setlocal

@echo Building bgfx w/ tools...
@set BGFX_GENIE_FLAGS=--with-tools
@set HOST=x86_64-pc-windows-msvc
@set TARGET=x86_64-pc-windows-msvc
@call "%~dp0..\bgfx-sys\build_msvc.cmd" >NUL

@echo Building example assets...
@set FLG_EXAMPLES=-i "%~dp0..\bgfx-sys\bgfx\src"
@set "SRC_EXAMPLES=%~dp0..\bgfx-sys\bgfx\examples"
@set "DST_EXAMPLES=%~dp0assets"
@set "PATH=%~dp0..\bgfx-sys\bgfx\.build\win64_vs2017\bin;%PATH%"
@call :build-assets 00-helloworld   || exit /b 1
@call :build-assets 01-cubes        || exit /b 1

@echo.
@echo Success!
@exit /b 0


:build-assets
@set "EXAMPLE=%~1"
@if not exist "%SRC_EXAMPLES%\%EXAMPLE%\*.sc" goto :skip-shaders
@for /f "" %%s in ('dir /b "%SRC_EXAMPLES%\%EXAMPLE%\fs_*.sc"') do @call :shaderc "%%s" Direct3D9  "--platform windows --profile ps_3_0 --type fragment -O 3" || exit /b 1
@for /f "" %%s in ('dir /b "%SRC_EXAMPLES%\%EXAMPLE%\fs_*.sc"') do @call :shaderc "%%s" Direct3D11 "--platform windows --profile ps_5_0 --type fragment -O 3" || exit /b 1
@for /f "" %%s in ('dir /b "%SRC_EXAMPLES%\%EXAMPLE%\fs_*.sc"') do @call :shaderc "%%s" Metal      "--platform osx     --profile metal  --type fragment -O 3" || exit /b 1
@for /f "" %%s in ('dir /b "%SRC_EXAMPLES%\%EXAMPLE%\fs_*.sc"') do @call :shaderc "%%s" OpenGL     "--platform linux   --profile 120    --type fragment -O 3" || exit /b 1
@for /f "" %%s in ('dir /b "%SRC_EXAMPLES%\%EXAMPLE%\fs_*.sc"') do @call :shaderc "%%s" OpenGLES   "--platform android                  --type fragment -O 3" || exit /b 1

@for /f "" %%s in ('dir /b "%SRC_EXAMPLES%\%EXAMPLE%\vs_*.sc"') do @call :shaderc "%%s" Direct3D9  "--platform windows --profile vs_3_0 --type vertex   -O 3" || exit /b 1
@for /f "" %%s in ('dir /b "%SRC_EXAMPLES%\%EXAMPLE%\vs_*.sc"') do @call :shaderc "%%s" Direct3D11 "--platform windows --profile vs_5_0 --type vertex   -O 3" || exit /b 1
@for /f "" %%s in ('dir /b "%SRC_EXAMPLES%\%EXAMPLE%\vs_*.sc"') do @call :shaderc "%%s" Metal      "--platform osx     --profile metal  --type vertex   -O 3" || exit /b 1
@for /f "" %%s in ('dir /b "%SRC_EXAMPLES%\%EXAMPLE%\vs_*.sc"') do @call :shaderc "%%s" OpenGL     "--platform linux   --profile 120    --type vertex   -O 3" || exit /b 1
@for /f "" %%s in ('dir /b "%SRC_EXAMPLES%\%EXAMPLE%\vs_*.sc"') do @call :shaderc "%%s" OpenGLES   "--platform android                  --type vertex   -O 3" || exit /b 1
:skip-shaders
@exit /b 0

:shaderc
shadercRelease %FLG_EXAMPLES% %~3 -f "%SRC_EXAMPLES%\%EXAMPLE%\%~nx1" -o "%DST_EXAMPLES%\%EXAMPLE%\%2\%~n1.bin"
@exit /b %ERRORLEVEL%
