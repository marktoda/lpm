# Local Package Manager

A package manager for local dependencies to enable automated cross-project integration.

## Building

### Build Dependencies

We recommend installing Rust through [rustup](https://www.rustup.rs/). If you don't already have `rustup`, you can install it like this:

- Linux:
  ```bash
  $ curl https://sh.rustup.rs -sSf | sh
  ```

- OSX:
  ```bash
  $ curl https://sh.rustup.rs -sSf | sh
  ```

- Windows:
  Make sure you have Visual Studio 2015 with C++ support installed. Next, download and run the `rustup` installer from
  https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe, start "VS2015 x64 Native Tools Command Prompt", and use the following command to install and set up the `msvc` toolchain:
  ```bash
  $ rustup default stable-x86_64-pc-windows-msvc
  ```

### Build from source

1. Clone
```bash
$ git clone git@github.com:marktoda/lpm.git
$ cd lpm

# build in release mode
$ cargo build --release
```

This produces an executable in the `./target/release` subdirectory.

### Running

As `lpm` is a CLI tool, the executable should be available in your executable PATH. Here are a couple ways to do this

1. Update PATH
```bash
# in your .bashprofile, .bashrc, .zshrc or equivalent
export PATH=/path/to/lpm/target/release/lpm:$PATH
```

2. Terminal alias
```bash
# in your .bashrc, .zshrc or equivalent
alias lpm="/path/to/lpm/target/release/lpm"
```

## Usage

`lpm` is an actively developed command-line tool, whose api may change over time. To get the most accurate, current api, run the below command:

```bash
$ lpm -h

USAGE:
    lpm [FLAGS] [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -v               Sets the level of verbosity
    -V, --version    Prints version information

SUBCOMMANDS:
    add       Add a new local package to the registry
    bundle    Bundle local dependencies for release of the given package.
    clear     Clear current package list
    help      Prints this message or the help of the given subcommand(s)
    list      List currently added packages
    update    Update all packages to introduce new code from its registered local dependencies
```

## Examples

The most common workflow for `lpm` is to stage a multi-dependency change locally to test the integration between components, without having to publish to npm. Below I will show an example of how I use it for development at BitGo:

```bash
$ lpm add $HOME/dev/BitGoJS/modules/statics
$ lpm add $HOME/dev/BitGoJS/modules/core
$ lpm add $HOME/dev/bitgo-account-lib
$ ... several other packages

$ lpm ls
... make sure that the registered packages look right and we're not missing anything

... develop some changes in various packages

$ lpm update

... run unit tests and integration tests on the components which are now locally integrated

$ lpm bundle BitGoJS

... push to server to run automated e2e tests
```

## Future Improvement

Some ideas for future improvement:
- *Custom package preparation*: All package are assumed to be prepared with `npm install ; npm run build`. `lpm` should support custom preparation scripts, maybe with an optional arg to `lpm add`
- *Sessions*: Ability to open / close sessions, reverting state to how it was before the session
- *Smart caching*: Use `crev-recursive-digest` crate or similar to get the version-hash of a package, and use it to avoid rebuilds if unchanged
- lots of cleanup, see inline TODOs
