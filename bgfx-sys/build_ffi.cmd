:: Copyright (c) 2019, Michael Rickert
:: License: http://opensource.org/licenses/ISC

:: Bindgen relies on some global installs, so this is made into it's own manual build step.
::  1)  LLVM for libclang.dll
::      Grab from http://releases.llvm.org/download.html
::      Last tested with http://releases.llvm.org/8.0.0/LLVM-8.0.0-win64.exe
::  2)  rustfmt
::      Just rustup component add rustfmt
::      Last tested with rustfmt 1.2.0-stable (09940a70 2019-03-27)

@pushd "%~dp0"

@set BINDGEN_VERSION=0.49.2

@type target\ffi\.crates.toml | findstr "bindgen" | findstr "%BINDGEN_VERSION%" >NUL && goto :skip-install-bindgen
cargo install bindgen --root target\ffi --force --version "%BINDGEN_VERSION%"
:skip-install-bindgen

::  FLAG                        WHY
:: --default-enum-style consts  We're just wrapping bgfx_sys enums in manually annotated enums anyways
:: --no-prepend-enum-name       All these rust style enums are prefixed with BGFX_ENUMNAME_ anyways, no need to do it twice.
:: --no-doc-comments            Tends to mis-associate many enum doc comments written in [Constant] /** Single line comment */ style.

target\ffi\bin\bindgen ^
    --whitelist-function bgfx_.* ^
    --whitelist-type     bgfx_.* ^
    --whitelist-var      BGFX_.* ^
    --no-prepend-enum-name ^
    --default-enum-style consts ^
    --no-doc-comments ^
    -o src\ffi_bgfx.rs ^
    bgfx\include\bgfx\c99\bgfx.h ^
    -- ^
    -Ibgfx\include ^
    -Ibx\include ^
    -include bgfx\c99\bgfx.h

:: While some BX_* constants exist, we intentionally avoid whitelisting them, as they're generally arch specific stuff.
:: Bindgen only generates them for one arch, meaning they're usually wrong.  The others don't seem useful.

@popd
