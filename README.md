gcc2clangd - fix-up GCC compilation databases for clangd
========================================================

The primary use-case for gcc2clangd occurs when you use want to use an editor
that uses clangd for auto-complete and source navigation but you compile your
code using gcc.

If the your code uses unusual compiler options, or if you are cross-compiling
there is a risk that clangd fails to index the code because the compilation
database contains unsupported options. In the authors experience this happens
most commonly when compiling the Linux kernel.

That's where gcc2clangd steps in. It allows you to post-process the compilation
database and automatically generate a `--target` option to teach clangd which
architecture you are compiling for. It will also strip out any command line
options that confuse clangd.

## Using gcc2clangd with the Linux kernel

The build system already provides support for generating a compilation database
as it runs. Try:

    make -j$(nproc) compile_commands.json
    gcc2clangd compile_commands.json

## Why doesn't your tool support -fan-exotic-option-nobody-knows-much-about?

Probably because I've never build code that uses
-fan-exotic-option-nobody-knows-much-about and didn't know much about it.

Pull requests are very welcome, provided they include comments on which
architectures are affected and include a clangd version number where it is
known to be a problem (both of which help with future maintenance).
