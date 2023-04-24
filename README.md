# `ogx`

> Build openGauss Extensions with Rust!

![cargo test --all](https://github.com/zombodb/ogx/workflows/cargo%20test%20--all/badge.svg)
[![crates.io badge](https://img.shields.io/crates/v/ogx.svg)](https://crates.io/crates/ogx)
[![docs.rs badge](https://docs.rs/ogx/badge.svg)](https://docs.rs/ogx)
[![Twitter Follow](https://img.shields.io/twitter/follow/zombodb.svg?style=flat)](https://twitter.com/zombodb)
[![Discord Chat](https://img.shields.io/discord/561648697805504526.svg)][Discord]


`ogx` is a framework for developing openGauss extensions in Rust and strives to be as idiomatic and safe as possible.

`ogx` supports openGauss v3.

## Key Features

- **A fully managed development environment with [`cargo-ogx`](./cargo-ogx/README.md)**
   + `cargo ogx new`: Create new extensions quickly
   + `cargo ogx init`: Install new (or register existing) openGauss installs
   + `cargo ogx run`: Run your extension and interactively test it in `gsql` (or `pgcli`)
   + `cargo ogx test`: Unit-test your extension across multiple openGauss versions
   + `cargo ogx package`: Create installation packages for your extension
   + More in the [`README.md`](./cargo-ogx/README.md)!
- **Target Multiple openGauss Versions**
   + Support openGauss v3 from the same codebase
   + Use Rust feature gating to use version-specific APIs
   + Seamlessly test against all versions
- **Automatic Schema Generation**
   + Implement extensions entirely in Rust
   + [Automatic mapping for many Rust types into openGauss](#mapping-of-opengauss-types-to-rust)
   + SQL schemas generated automatically (or manually via `cargo ogx schema`)
   + Include custom SQL with `extension_sql!` & `extension_sql_file!`
- **Safety First**
   + Translates Rust `panic!`s into openGauss `ERROR`s that abort the transaction, not the process
   + Memory Management follows Rust's drop semantics, even in the face of `panic!` and `elog(ERROR)`
   + `#[og_guard]` procedural macro to ensure the above
   + openGauss `Datum`s are `Option<T> where T: FromDatum`
      - `NULL` Datums are safely represented as `Option::<T>::None`
- **First-class UDF support**
   + Annotate functions with `#[og_extern]` to expose them to openGauss
   + Return `ogx::iter::SetOfIterator<'a, T>` for `RETURNS SETOF`
   + Return `ogx::iter::TableIterator<'a, T>` for `RETURNS TABLE (...)`
   + Create trigger functions with `#[og_trigger]`
- **Easy Custom Types**
   + `#[derive(OgType)]` to use a Rust struct as a openGauss type
      - By default, represented as a CBOR-encoded object in-memory/on-disk, and JSON as human-readable
      - Provide custom in-memory/on-disk/human-readable representations
   + `#[derive(OgEnum)]` to use a Rust enum as a openGauss enum
   + Composite types supported with the `ogx::composite_type!("Sample")` macro
- **Server Programming Interface (SPI)**
   + Safe access into SPI
   + Transparently return owned Datums from an SPI context
- **Advanced Features**
   + Safe access to openGauss' `MemoryContext` system via `ogx::OgMemoryContexts`
   + Executor/planner/transaction/subtransaction hooks
   + Safely use openGauss-provided pointers with `ogx::OgBox<T>` (akin to `alloc::boxed::Box<T>`)
   + `#[og_guard]` proc-macro for guarding `extern "C"` Rust functions that need to be passed into openGauss
   + Access openGauss' logging system through `eprintln!`-like macros
   + Direct `unsafe` access to large parts of openGauss internals via the `ogx::pg_sys` module
   + New features added regularly!

## System Requirements

- A Rust toolchain: `rustc`, `cargo`, and `rustfmt`. The recommended way to get these is from https://rustup.rs †
- `git`
- `libclang` 5.0 or greater (required by bindgen)
   - Ubuntu: `apt install libclang-dev` or `apt install clang`
   - RHEL: `yum install clang`
- `tar`
- `bzip2`
- GCC 7 or newer
- [openGauss's build dependencies](https://gitee.com/opengauss/openGauss-server#%E6%93%8D%E4%BD%9C%E7%B3%BB%E7%BB%9F%E5%92%8C%E8%BD%AF%E4%BB%B6%E4%BE%9D%E8%B5%96%E8%A6%81%E6%B1%82) ‡

 † OGX has no MSRV policy, thus may require the latest stable version of Rust, available via Rustup

 ‡ A local openGauss server installation is not required. `cargo ogx` can download and compile openGauss versions on its own.


## Getting Started


First install the `cargo-ogx` sub-command and initialize the development environment:

```bash
cargo install --locked cargo-ogx
cargo ogx init
```

The `init` command downloads openGauss version v3 compiles them to `~/.ogx/`, and runs `initdb`. It's also possible to use an existing (user-writable) openGauss install, or install a subset of versions, see the [`README.md` of `cargo-ogx` for details](cargo-ogx/README.md#first-time-initialization).

```bash
cargo ogx new my_extension
cd my_extension
```

This will create a new directory for the extension crate.

```
$ tree 
.
├── Cargo.toml
├── my_extension.control
├── sql
└── src
    └── lib.rs

2 directories, 3 files
```

The new extension includes an example, so you can go ahead and run it right away.

```bash
cargo ogx run og3
```

This compiles the extension to a shared library, copies it to the specified openGauss installation, starts that openGauss instance and connects you to a database named the same as the extension.

Once `cargo-ogx` drops us into `gsql` we can load the extension and do a SELECT on the example function.

```sql
my_extension=# CREATE EXTENSION my_extension;
CREATE EXTENSION

my_extension=# SELECT hello_my_extension();
 hello_my_extension
---------------------
 Hello, my_extension
(1 row)
```

For more details on how to manage ogx extensions see [Managing ogx extensions](./cargo-ogx/README.md).

## Upgrading

You can upgrade your current `cargo-ogx` installation by passing the `--force` flag
to `cargo install`:

```bash
cargo install --force --locked cargo-ogx
```

As new openGauss versions are supported by `ogx`, you can re-run the `ogx init` process to download and compile them:

```bash
cargo ogx init
```

### Mapping of openGauss types to Rust

openGauss Type | Rust Type (as `Option<T>`)
--------------|-----------
`bytea` | `Vec<u8>` or `&[u8]` (zero-copy)
`text` | `String` or `&str` (zero-copy)
`varchar` | `String` or `&str` (zero-copy) or `char`
`"char"` | `i8`
`smallint` | `i16`
`integer` | `i32`
`bigint` | `i64`
`oid` | `u32`
`real` | `f32`
`double precision` | `f64`
`bool` | `bool`
`json` | `ogx::Json(serde_json::Value)`
`jsonb` | `ogx::JsonB(serde_json::Value)`
`date` | `ogx::Date`
`time` | `ogx::Time`
`timestamp` | `ogx::Timestamp`
`time with time zone` | `ogx::TimeWithTimeZone`
`timestamp with time zone` | `ogx::TimestampWithTimeZone`
`anyarray` | `ogx::AnyArray`
`anyelement` | `ogx::AnyElement`
`box` | `ogx::pg_sys::BOX`
`point` | `ogx::ogx_sys::Point`
`tid` | `ogx::pg_sys::ItemPointerData`
`cstring` | `&std::ffi::CStr`
`inet` | `ogx::Inet(String)` -- TODO: needs better support
`numeric` | `ogx::Numeric(String)` -- TODO: needs better support
`void` | `()`
`ARRAY[]::<type>` | `Vec<Option<T>>` or `ogx::Array<T>` (zero-copy)
`NULL` | `Option::None`
`internal` | `ogx::OgBox<T>` where `T` is any Rust/openGauss struct
`uuid` | `ogx::Uuid([u8; 16])`

There are also `IntoDatum` and `FromDatum` traits for implementing additional type conversions,
along with `#[derive(OgType)]` and `#[derive(OgEnum)]` for automatic conversion of
custom types.

## Digging Deeper

 - [cargo-ogx sub-command](cargo-ogx/)
 - [Custom Types](ogx-examples/custom_types/)
 - [openGauss Operator Functions and Operator Classes/Families](ogx-examples/operators/)
 - [Shared Memory Support](ogx-examples/shmem/)
 - [various examples](ogx-examples/)

## Caveats & Known Issues

There's probably more than are listed here, but a primary things of note are:

 - Threading is not really supported.  Postgres is strictly single-threaded.  As such, if you do venture into using threads, those threads **MUST NOT** call *any* internal Postgres function, or otherwise use any Postgres-provided pointer.  There's also a potential problem with Postgres' use of `sigprocmask`.  This was being discussed on the -hackers list, even with a patch provided, but the conversation seems to have stalled (https://www.postgresql.org/message-id/flat/5EF20168.2040508%40anastigmatix.net#4533edb74194d30adfa04a6a2ce635ba).
 - How to correctly interact with Postgres in an `async` context remains unexplored.
 - `ogx` wraps a lot of `unsafe` code, some of which has poorly-defined safety conditions. It may be easy to induce illogical and undesirable behaviors even from safe code with `ogx`, and some of these wrappers may be fundamentally unsound. Please report any issues that may arise.
 - Not all of Postgres' internals are included or even wrapped.  This isn't due to it not being possible, it's simply due to it being an incredibly large task.  If you identify internal Postgres APIs you need, open an issue and we'll get them exposed, at least through the `ogx::pg_sys` module.
 - Windows is not supported.  It could be, but will require a bit of work with `cargo-ogx` and figuring out how to compile `ogx`'s "cshim" static library.
 - Sessions started before `ALTER EXTENSION my_extension UPDATE;` will continue to see the old version of `my_extension`. New sessions will see the updated version of the extension.
 - `ogx` is used by many "in production", but it is not "1.0.0" or above, despite that being recommended by SemVer for production-quality software. This is because there are many unresolved soundness and ergonomics questions that will likely require breaking changes to resolve, in some cases requiring cutting-edge Rust features to be able to expose sound interfaces. While a 1.0.0 release is intended at some point, it seems prudent to wait until it seems like a 2.0.0 release would not be needed the next week and the remaining questions can be deferred.

## TODO

There's a few things on our immediate TODO list

 - Automatic extension schema upgrade scripts, based on diffs from a previous git tag and HEAD.  Likely, this
will be built into the `cargo-ogx` subcommand and make use of https://github.com/zombodb/postgres-parser.
 - More examples -- especially around memory management and the various derive macros `#[derive(OgType/Enum)]`


## Feature Flags
OGX has optional feature flags for Rust code that do not involve configuring the version of Postgres used,
but rather extend additional support for other kinds of Rust code. These are not included by default.

## Safety of ogx
Documentation for invariants that `ogx` relies on for the soundness of its Rust interface,
or ways that `ogx` compensates for assumed non-invariants, or just notes about
the quirks of openGauss that have been discovered.

Specific functions will have their safety conditions documented on them,
so this document is only useful for describing higher-level concepts.

### openGauss

Quirks specific to openGauss.

#### Memory Allocation

The `palloc*` family of functions may throw a openGauss error but will not return `nullptr`.

### Rust

Quirks specific to Rust that specifically inform the design of this crate and not, say,
"every single crate ever".

#### Destructors Are Not Guaranteed And `sig{set,long}jmp` Is Weird

Rust does not guarantee that a `Drop::drop` implementation, even if it is described, will actually
be run, due to the ways control flow can be interrupted before the destructor starts or finishes.
Indeed, Drop implementations can be precisely a source of such problems if they are "non-trivial".
Rust control flow has to be independently safe from openGauss control flow to keep openGauss safe from Rust,
and Rust safe from openGauss.

Accordingly, it should be noted that Rust isn't really designed with `sigsetjmp` or `siglongjmp` in mind,
even though they are used in this crate and work well enough at making Rust more manageable
in the face of the various machinations that openGauss may get up to.

## Contributing

We are most definitely open to contributions of any kind.  Bug Reports, Feature Requests, Documentation,
and even [sponsorships](https://github.com/sponsors/eeeebbbbrrrr).

If you'd like to contribute code via a Pull Request, please make it against our `develop` branch.  The `master` branch is meant to represent what is currently available on crates.io.

Providing wrappers for Postgres' internals is not a straightforward task, and completely wrapping it is going
to take quite a bit of time.  `ogx` is generally ready for use now, and it will continue to be developed as
time goes on.  Your feedback about what you'd like to be able to do with `ogx` is greatly appreciated.


## License

```
Portions Copyright 2019-2021 ZomboDB, LLC.  
Portions Copyright 2021-2022 Technology Concepts & Design, Inc. <support@tcdi.com>. 
All rights reserved.
Use of this source code is governed by the MIT license that can be found in the LICENSE file.
```

[Discord]: https://discord.gg/hPb93Y9
[timecrate]: https://crates.io/crates/time
