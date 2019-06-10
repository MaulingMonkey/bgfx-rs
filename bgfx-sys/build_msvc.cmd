:: Copyright (c) 2019, Michael Rickert
:: License: http://opensource.org/licenses/ISC

:: We have two conflicting goals:
::  1) Genie and MSBuild need a sane vcvars*.bat context for inferring the correct Windows SDK Version etc.
::  2) We'd like to allow building multiple architectures (x86 or AMD64), despite vcvars*.bat limiting cargo to a single arch.
:: Additionally:
::  3) We'd like `cargo build` in a vanilla cmd.exe prompt to just work (tm) like it does for vanilla Rust.
::
:: Why build.cmd instead of doing this in build.rs?
::  1) Easy inheritence of env vars, without trying to parse the output of `vcvars*.bat && set`
::  2) `Command::new` 's built-in escaping messes with our ability to do a smart one liner using cmd.exe's magic escaping like:
::      cmd /c "call "...\vcvars64.bat" && MSBuild "path\to\sln""



:: Autoconfigure
@call :infer-vcvarsall-arch     || exit /b 1
@call :find-msvc                || exit /b 1
@call :infer-bgfx-vs20nn        || exit /b 1
@call :infer-bgfx-vsconfig      || exit /b 1
@call :infer-bgfx-vsplatform    || exit /b 1
@call :infer-bgfx-winNN         || exit /b 1

:: Build
@pushd "%~dp0\bgfx"
..\bx\tools\bin\windows\genie.exe --with-dynamic-runtime %BGFX_VS20NN%                                              || exit /b 1
MSBuild.exe /p:Configuration=%BGFX_VSCONFIG% /p:Platform=%BGFX_VSPLATFORM% ".build\projects\%BGFX_VS20NN%\bgfx.sln" || exit /b 1
@popd

:: Rust settings
@echo cargo:rustc-link-lib=static=bx%BGFX_VSCONFIG%
@echo cargo:rustc-link-lib=static=bimg%BGFX_VSCONFIG%
@echo cargo:rustc-link-lib=static=bgfx%BGFX_VSCONFIG%
@echo cargo:rustc-link-lib=gdi32
@echo cargo:rustc-link-lib=user32
@echo cargo:rustc-link-search=native=%~dp0\bgfx\.build\%BGFX_WINNN%_%BGFX_VS20NN%\bin

:: Exit Successfully
@exit /b 0



:find-msvc
@where MSBuild.exe && exit /b 0
:: VS150COMNTOOLS only defined in developer command prompts, so search default install locations for VS2017
@if defined ProgramFiles(x86) for /d %%e in ("%ProgramFiles(x86)%\Microsoft Visual Studio\2017\*") do call :find-msvc-2017 %%e && exit /b 0
@if defined ProgramFiles      for /d %%e in      ("%ProgramFiles%\Microsoft Visual Studio\2017\*") do call :find-msvc-2017 %%e && exit /b 0
:: VS140COMNTOOLS defined in general cmd.exe instances
@if defined VS140COMNTOOLS @call :find-msvc-at "%VS140COMNTOOLS%\..\..\VC\vcvarsall.bat" && exit /b 0
@if defined VS120COMNTOOLS @call :find-msvc-at "%VS120COMNTOOLS%\..\..\VC\vcvarsall.bat" && exit /b 0
@echo Couldn't find MSVC installation to build BGFX with
@exit /b 1

:find-msvc-2017
:: for /d %%e ("dir\*") do ... seems to really not want to quote the path, even if I use "%%e", so we need to abuse %*
@set "MSVC_EDITION=%*"
@call :find-msvc-path "%MSVC_EDITION%\VC\Auxiliary\Build\vcvarsall.bat"
@exit /b %ERRORLEVEL%

:find-msvc-path
@set "VCVARSALL_BAT=%~1"
@if not exist "%VCVARSALL_BAT%" exit /b 1
@call "%VCVARSALL_BAT%" %VCVARSALL_ARCH%
@exit /b %ERRORLEVEL%

:infer-bgfx-vs20nn
@set BGFX_VS20NN=
@if not defined BGFX_VS20NN if /i "%VisualStudioVersion%" == "12.0" set BGFX_VS20NN=vs2013
@if not defined BGFX_VS20NN if /i "%VisualStudioVersion%" == "14.0" set BGFX_VS20NN=vs2015
@if not defined BGFX_VS20NN if /i "%VisualStudioVersion%" == "15.0" set BGFX_VS20NN=vs2017
@if not defined BGFX_VS20NN echo :infer-bgfx-vs20nn: Unrecognized VisualStudioVersion %VisualStudioVersion%&& exit /b 1
@exit /b 0

:infer-bgfx-vsconfig
:: Rust currently links against the Release MSVC runtimes unconditionally, so we need to as well.
:: If https://github.com/rust-lang/rust/issues/39016 has been resolved, consider basing this off of... %PROFILE% ?
@set BGFX_VSCONFIG=Release
@exit /b 0

:infer-bgfx-vsplatform
@set BGFX_VSPLATFORM=
@if not defined BGFX_VSPLATFORM if /i "%TARGET:~0,7%" == "x86_64-" set BGFX_VSPLATFORM=x64
@if not defined BGFX_VSPLATFORM if /i "%TARGET:~0,5%" == "i686-"   set BGFX_VSPLATFORM=Win32
@if not defined BGFX_VSPLATFORM echo :infer-bgfx-vsplatform: Unrecognized TARGET architecture %TARGET%&& exit /b 1
@exit /b 0

:infer-bgfx-winNN
@set BGFX_WINNN=
@if not defined BGFX_WINNN if /i "%TARGET:~0,7%" == "x86_64-" set BGFX_WINNN=win64
@if not defined BGFX_WINNN if /i "%TARGET:~0,5%" == "i686-"   set BGFX_WINNN=win32
@if not defined BGFX_WINNN echo :infer-bgfx-winNN: Unrecognized TARGET architecture %TARGET%&& exit /b 1
@exit /b 0

:infer-vcvarsall-arch
@set VCVARSALL_ARCH=
@if not defined VCVARSALL_ARCH if /i "%HOST:~0,7%" == "x86_64-" if /i "%TARGET:~0,7%" == "x86_64-" set VCVARSALL_ARCH=amd64
@if not defined VCVARSALL_ARCH if /i "%HOST:~0,5%" == "i686-"   if /i "%TARGET:~0,5%" == "i686-"   set VCVARSALL_ARCH=x86
@if not defined VCVARSALL_ARCH if /i "%HOST:~0,7%" == "x86_64-" if /i "%TARGET:~0,5%" == "i686-"   set VCVARSALL_ARCH=amd64_x86
@if not defined VCVARSALL_ARCH if /i "%HOST:~0,5%" == "i686-"   if /i "%TARGET:~0,7%" == "x86_64-" set VCVARSALL_ARCH=x86_amd64
@if not defined VCVARSALL_ARCH echo :find-vcvars-bat: Unrecognized HOST (%HOST%) or TARGET (%TARGET%) architecture&& exit /b 1
@exit /b 0
