# How to cross-compile

These instructions assume Arch Linux x86\_64 to Raspberry Pi OS aarch64. Change your toolchain names as required. I offer no support for this. Feel free to ask if it does not work, but don't get your hopes up, the whole thing is a mess.  
These instructions use bash globbing syntax.

## Requirements:

Working host and target environments. Make sure both are up to date.

## Steps:

1. Install the `aarch64-linux-gnu-gcc` package and all its dependencies.
2. Check the version of glibc installed on your target by running `/usr/lib/aarch64-linux-gnu/libc.so.6`
3. Go to `https://archive.archlinux.org/packages/a/aarch64-linux-gnu-glibc/` and download and install on the host the version you found above. If there isn't one, try the latest but you may be out of luck.
4. In the crate root execute `CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc cargo build --target aarch64-unknown-linux-gnu`. Add switches as necessary, as you normally would.
5. If you see errors, scroll all the way to the beginning of the list. There should be a list of missing libraries. Copy them from `/usr/lib/aarch64-linux-gnu/` of the target to `/usr/aarch64-linux-gnu/lib{64,}/` of the host. Go back to step 4.
6. You should now have a working binary in `target/aarch64-unknown-linux-gnu/debug`. Copy it and all required files (such as the database) to your target system and try executing it.

## Possible issues:

If you get errors about `GLIBC` not being the right version repeat steps 2 and 3. They need to match.
