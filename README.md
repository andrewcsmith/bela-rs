# bela-rs

Safe Rust wrapper for the Bela microcontroller API.

## Setup

For this, you will need a [Bela microcontroller](http://bela.io/) with
compiled libbela.so libraries, the [Rust programming
language](https://rustup.rs/), and the relevant ABI files.

You can download the [arm-linux-gnueabihf for all platforms at the linaro
site](https://releases.linaro.org/components/toolchain/binaries/latest-5/arm-linux-gnueabihf/),
or likely through your package manager of choice.

### Dependencies

It's possible to link this crate against either [padenot's bela-sys crate](https://github.com/padenot/bela-sys) or [andrewcsmith/bela-sys](https://github.com/andrewcsmith/bela-sys). The difference between the two is mainly in that padenot uses a vendored version of the bela.rs and header files, while the andrewcsmith version generates its own headers using `bindgen` and a local copy of all the relevant header files. This is significantly more complicated to set up.

padenot/bela-sys is tested on OSX and Linux, while andrewcsmith/bela-sys is tested on Windows 10 Professional.

## Design

bela-rs aims to be a safe wrapper around the core Bela functionality, but
there are a few opinionated design choices to take advantage of specific
capabilities of Rust. The first of these is *runtime guarantees* about the
`userData void*` passed to every call of `render`, `setup`, or `cleanup`. The
global function calls to the Bela are managed by a `Bela` struct.

Second, the lifecycle functions (`render`, `setup`, `cleanup`) are not
globally defined functions, but rather are defined as closures that are
passed to every call in the lifecycle. This allows the programmer to use
features in closures such as capturing outside variables, and mutating state
on each call.

Third, the auxiliary tasks are also closures, and are separated into callback
functions and arguments to be passed to the first call.

## Example

```rust
// Short extract of code from examples/hello.rs, where phasor is some arbitrary
// data that is passed to each render, setup, and cleanup call.
let user_data = AppData::new(phasor, &mut render, Some(&mut setup), Some(&mut cleanup));
let mut settings = InitSettings::default();
// The .run call blocks until interrupted, returning a Result
Bela::new(user_data).run(&mut settings) 
```
