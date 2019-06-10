bgfx-rs [![travis-ci status](https://travis-ci.org/MaulingMonkey/bgfx-rs.svg?branch=master)](https://travis-ci.org/MaulingMonkey/bgfx-rs) [![appveyor status](https://ci.appveyor.com/api/projects/status/github/MaulingMonkey/bgfx-rs?branch=master&svg=true)](https://ci.appveyor.com/project/MaulingMonkey/bgfx-rs/branch/master)
=======

Rust wrapper around [bgfx], providing a clean, safe API for rust applications.

**Note:** This is a fork of [rhoot/bgfx-rs](https://github.com/rhoot/bgfx-rs), which is no longer maintained.
This may also end up unmaintained, but let's give it a shot anyways.

### Quick Reference

* License:  [ISC]
* Documentation:  [rhoot/bgfx-rs][docs]



Platforms
---------

| OS            | Target                    | Requirements                                                      | Recommended   |
|-------------- |-------------------------- |------------------------------------------------------------------ |-------------- |
| Windows       | *-pc-windows-msvc [1]     | [Rust](https://rustup.rs/), [VS2017, VS2015, or VS2013](https://visualstudio.microsoft.com/vs/older-downloads/) with the standard C++ tools [2].  | [VS Code](https://code.visualstudio.com/)
| Windows       | *-pc-windows-gnu [1]      | [Rust](https://rustup.rs/), MinGW?, [Some GnuWin32 Utilities](https://bkaradzic.github.io/bgfx/build.html#windows)                                | [VS Code](https://code.visualstudio.com/)
| Linux         | *-unknown-linux-gnu [1]   | [Rust](https://rustup.rs/), G++ 4.8?, Make, [OpenGL and X11](https://bkaradzic.github.io/bgfx/build.html#linux) dev packages                      |
| Windows UWP   |                           | Rust and BGFX have support, needs implementing though.            |
| Android       |                           | Rust and BGFX have support, needs implementing though.            |
| iOS           |                           | Rust and BGFX have support, needs implementing though.            |
| OS X          |                           | Rust and BGFX have support, needs implementing though.            |
| Windows Phone |                           | BGFX has support, but Microsoft has sunset this platform.         |

1) x86_64 or i686
2) The x86 compiler may require an opt-in individual component in some VS versions.
3) See also [bgfx's build docs](https://bkaradzic.github.io/bgfx/build.html).  `bx`, `bimg`, and `genie` dependencies can be ignored as they're handled via submodules.



Quick Start
-----------

### Debug Examples (VS Code, Windows)

1) If you haven't already, install [VS2017](https://visualstudio.microsoft.com/vs/older-downloads/) w/ Desktop C++ Tools, [VS Code](https://code.visualstudio.com/), and [Git](https://git-scm.com/)
2) Open a `cmd` prompt
    - Run `git clone --recurse-submodules https://github.com/MaulingMonkey/bgfx-rs.git` to get the full source code, or run `git submodule update --init` if you already had a partial checkout.
    - Run `code bgfx-rs` to open VS Code in the right folder.
3) Consider installing workspace recommended extensions (`Ctrl+Shift+X` to open the Extensions tab)
4) Press `F5` to `Start Debugging`
5) Press `Enter` to select the default example, `00-helloworld`, when VS Code prompts you
6) Profit!  Set breakpoints, step through code, inspect variables, etc.

### Run Examples (Command Line, Multi-Platform)

Just use the regular cargo commands:
```
cargo run --example 00-helloworld
cargo run --example 01-cubes
```

**OSX Note:** There is currently no really clean way to exit the examples in
OSX, and closing the window may in fact cause a crash. This is due to
limitations in [glutin][glutin] (specifically [#468] and [#520]). This only
effects the examples, and not the crate itself. The best way of closing them
is to simply `Ctrl-C` in the console.



License
-------
Copyright (c) 2015-2016, Johan Sköld

Permission to use, copy, modify, and/or distribute this software for any  
purpose with or without fee is hereby granted, provided that the above  
copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES  
WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF  
MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR  
ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES  
WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN  
ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF  
OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.


[#468]:   https://github.com/tomaka/glutin/issues/468   "tomaka/glutin #468"
[#520]:   https://github.com/tomaka/glutin/issues/520   "tomaka/glutin #520"
[bgfx]:   https://github.com/bkaradzic/bgfx             "bgfx"
[docs]:   https://rhoot.github.io/bgfx-rs/bgfx/         "Bindings documentation"
[glutin]: https://github.com/tomaka/glutin              "glutin"
[rhoot/bgfx-rs]: https://github.com/rhoot/bgfx-rs       "rhoot/bgfx-rs"
[ISC]:      LICENSE                                     "ISC License"
