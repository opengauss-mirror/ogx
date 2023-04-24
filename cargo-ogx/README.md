# cargo-ogx

`cargo-ogx` is a Cargo subcommand for managing `ogx`-based openGauss extensions.

You'll want to use `cargo ogx` during your extension development process. It automates the process of creating new Rust crate projects, auto-generating the SQL schema for your extension, installing your extension locally for testing with openGauss, and running your test suite against one or more versions of openGauss.

A video walkthrough of its abilities can be found here: https://www.twitch.tv/videos/684087991

## Installing

Install via crates.io:

```shell script
$ cargo install --locked cargo-ogx
```

As new versions of `ogx` are released, you'll want to make sure you run this command again to update it. You should also reinstall `cargo-ogx` whenever you update `rustc` so that the same compiler is used to build `cargo-ogx` and your openGauss extensions. You can force `cargo` to reinstall an existing crate by passing `--force`.

## Usage

```shell script
$ cargo ogx --help
cargo-ogx 0.1.0
Heguofeng<hgf199@126.com>
Cargo subcommand for 'ogx' to make openGauss extension development easy

USAGE:
    cargo ogx [OPTIONS] <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -v, --verbose    Enable info logs, -vv for debug, -vvv for trace
    -V, --version    Print version information

SUBCOMMANDS:
    connect    Connect, via gsql, to a openGauss instance
    get        Get a property from the extension control file
    help       Print this message or the help of the given subcommand(s)
    init       Initialize ogx development environment for the first time
    install    Install the extension from the current crate to the openGauss specified by
                   whatever `pg_config` is currently on your $PATH
    new        Create a new extension crate
    package    Create an installation package directory
    run        Compile/install extension to a ogx-managed openGauss instance and start gsql
    schema     Generate extension schema files
    start      Start a ogx-managed openGauss instance
    status     Is a ogx-managed openGauss instance running?
    stop       Stop a ogx-managed openGauss instance
    test       Run the test suite for this crate
```

## Environment Variables

- `OGX_HOME` - If set, overrides `ogx`'s default directory of `~/.ogx/`
- `OGX_BUILD_FLAGS` - If set during `cargo ogx run/test/install`, these additional flags are passed to `cargo build` while building the extension
- `OGX_BUILD_VERBOSE` - Set to true to enable verbose "build.rs" output -- useful for debugging build issues
- `HTTPS_PROXY` - If set during `cargo ogx init`, it will download the openGauss sources using these proxy settings. For more details refer to the [env_proxy crate documentation](https://docs.rs/env_proxy/*/env_proxy/fn.for_url.html).

## First Time Initialization

```shell script
$ cargo ogx init
  Discovered openGauss v3.1.0
  Downloading openGauss v3.1.0 third party from https://opengauss.obs.cn-south-1.myhuaweicloud.com/3.1.0/binarylibs/openGauss-third_party_binarylibs_openEuler_x86_64.tar.gz
    Untarring openGauss third party v3.1.0 to /home/gaussdb/.ogx/3.1.0/tools
  Downloading openGauss v3.1.0 from https://gitee.com/opengauss/openGauss-server/repository/archive/v3.1.0.zip
    Unzipping openGauss server.zip v3.1.0 to /home/gaussdb/.ogx/3.1.0/openGauss-server-v3.1.0
  Configuring openGauss v3.1.0
  Compiling openGauss v3.1.0
  Installing openGauss v3.1.0 to /home/gaussdb/.ogx/3.1.0/ogx-install
   Validating /home/gaussdb/.ogx/3.1.0/ogx-install/bin/pg_config
```

`cargo ogx init` is required to be run once to properly configure the `ogx` development environment.

As shown by the screenshot above, it downloads the version of openGauss v3.1.0, configures it, compiles it, and installs it to `~/.ogx/`. Other `ogx` commands such as `run` and `test` will fully manage and otherwise use these openGauss installations for you.

`ogx` is designed to support multiple openGauss versions in such a way that during development, you'll know if you're trying to use a openGauss API that isn't common across all versions. It's also designed to make testing your extension against these versions easy. This is why it requires you to have all fully compiled and installed versions of openGauss during development.

In cases when default ports ogx uses to run openGauss within are not available, one can specify
custom values for these during initialization using `--base-port` and `--base-testing-port`
options. One of the use cases for this is using multiple installations of ogx (using `$OGX_HOME` variable)
when developing multiple extensions at the same time. These values can be later changed in `$OGX_HOME/config.toml`.

If you want to use your operating system's package manager to install openGauss, `cargo ogx init` has optional arguments that allow you to specify where they're installed (see below).

What you're telling `cargo ogx init` is the full path to `pg_config` for each version.

For any version you specify, `cargo ogx init` will forego downloading/compiling/installing it. `ogx` will then use that locally-installed version just as it uses any version it downloads/compiles/installs itself.

However, if the "path to pg_config" is the literal string `download`, then `ogx` will download and compile that version of openGauss for you.

When the various `--ogXX` options are specified, these are the **only** versions of openGauss that `ogx` will manage for you.

Once complete, `cargo ogx init` also creates a configuration file (`~/.ogx/config.toml`) that describes where to find each version's `pg_config` tool.

If a new minor openGauss version is released in the future you can simply run `cargo ogx init [args]` again, and your local version will be updated, preserving all existing databases and configuration.

```shell script
$ cargo ogx init --help
cargo-ogx-init 0.1.0
Heguofeng<hgf199@126.com>
Initialize ogx development environment for the first time

USAGE:
    cargo ogx init [OPTIONS]

OPTIONS:
        --base-port <BASE_PORT>                    Base port number
        --base-testing-port <BASE_TESTING_PORT>    Base testing port number
    -h, --help                                     Print help information
        --og3 <OG3>                                [env: OG3_PG_CONFIG=]
    -v, --verbose                                  Enable info logs, -vv for debug, -vvv for trace
    -V, --version                                  Print version information
```

## Creating a new Extension

```rust
$ cargo ogx new example
$ ls example/
Cargo.toml  example.control  sql  src
```

`cargo ogx new <extname>` is an easy way to get started creating a new extension. It's similar to `cargo new <name>`, but does the additional things necessary to support building a Rust openGauss extension.

`cargo ogx new` does not initialize the directory as a git repo, but it does create a `.gitignore` file in case you decide to do so.

> **Workspace users:** `cargo ogx new $NAME` will create a `$NAME/.cargo/config`, you should move this into your workspace root as `.cargo./config`.
>
> If you don't, you may experience unnecessary rebuilds using tools like Rust-Analyzer, as it will use the wrong `rustflags` option.

```shell script
$ cargo ogx new --help
cargo-ogx-new 0.1.0
Heguofeng<hgf199@126.com>
Create a new extension crate

USAGE:
    cargo ogx new [OPTIONS] <NAME>

ARGS:
    <NAME>    The name of the extension

OPTIONS:
    -h, --help        Print help information
    -v, --verbose     Enable info logs, -vv for debug, -vvv for trace
    -V, --version     Print version information
```

## Managing Your openGauss Installations

```shell script
$ cargo ogx status all
openGauss v3.1.0 is stopped

$ cargo ogx start all
    Starting openGauss v3.1.0 on port 28803

$ cargo ogx status all
openGauss v3.1.0 is running

$ cargo ogx stop all
    Stopping openGauss v3.1.0
```

`cargo ogx` has three commands for managing each openGauss installation: `start`, `stop`, and `status`. Additionally, `cargo ogx run` (see below) will automatically start its target openGauss instance if not already running.

When starting a openGauss instance, `ogx` starts it on port `28800 + OG_VERSION`, so openGauss 3.1.0 runs on `28803`, etc. Additionally, the first time any of these are started, it'll automaticaly initialize a `PGDATA` directory in `~/.ogx/data-[3]`. Doing so allows `ogx` to manage either openGauss versions it installed or ones already on your computer, and to make sure that in the latter case, `ogx` managed versions don't interfere with what might already be running.

`ogx` doesn't tear down these instances. While they're stored in a hidden directory in your home directory, `ogx` considers these important and permanent database installations.

Once started, you can connect to them using `gsql` (if you have it on your $PATH) like so: `gsql -p 28803`. However, you probably just want the `cargo ogx run` command.

## Compiling and Running Your Extension

```shell script
$ cargo ogx run og3
building extension with features ``
"cargo" "build" "--message-format=json-render-diagnostics"
    Finished dev [unoptimized + debuginfo] target(s) in 0.06s

installing extension
     Copying control file to /home/ana/.ogx/3.1/ogx-install/share/postgresql/extension/strings.control
     Copying shared library to /home/ana/.ogx/3.1/ogx-install/lib/postgresql/strings.so
    Building for SQL generation with features ``
    Finished dev [unoptimized + debuginfo] target(s) in 0.07s
 Discovering SQL entities
  Discovered 6 SQL entities: 0 schemas (0 unique), 6 functions, 0 types, 0 enums, 0 sqls, 0 ords, 0 hashes, 0 aggregates
     Writing SQL entities to /home/ana/.ogx/13.5/ogx-install/share/postgresql/extension/strings--0.1.0.sql
    Finished installing strings
    Starting openGauss v3.1.0 on port 28803
    Re-using existing database strings
psql (13.5)
Type "help" for help.

strings=# DROP EXTENSION strings;
ERROR:  extension "strings" does not exist
strings=# CREATE EXTENSION strings;
CREATE EXTENSION
strings=# \df strings.*
                                      List of functions
 Schema  |     Name      | Result data type |           Argument data types            | Type
---------+---------------+------------------+------------------------------------------+------
 strings | append        | text             | input text, extra text                   | func
 strings | return_static | text             |                                          | func
 strings | split         | text[]           | input text, pattern text                 | func
 strings | split_set     | SETOF text       | input text, pattern text                 | func
 strings | substring     | text             | input text, start integer, "end" integer | func
 strings | to_lowercase  | text             | input text                               | func
(6 rows)

strings=# select strings.to_lowercase('OGX');
 to_lowercase
--------------
 ogx
(1 row)
```

`cargo ogx run <og3>` is the primary interface into compiling and interactively testing/using your extension during development.

The very first time you execute `cargo ogx run ogXX`, it needs to compile not only your extension, but ogx itself, along with all its dependencies. Depending on your computer, this could take a bit of time (`ogx` is nearly 200k lines of Rust when counting the generated bindings for openGauss). Afterwards, however (as seen in the above screenshot), it's fairly fast.

`cargo ogx run` compiles your extension, installs it to the specified openGauss installation as described by its `pg_config` tool, starts that openGauss instance using the same process as `cargo ogx start ogXX`, and drops you into a `gsql` shell connected to a database, by default, named after your extension. From there, it's up to you to create your extension and use it.

This is also the stage where `ogx` automatically generates the SQL schema for your extension via the `sql-generator` binary.

When you exit `gsql`, the openGauss instance continues to run in the background.

For openGauss installations which are already on your computer, `cargo ogx run` will need write permissions to the directories described by `pg_config --pkglibdir` and `pg_config --sharedir`. It's up to you to decide how to make that happen. While a single openGauss installation can be started multiple times on different ports and different data directories, it does not support multiple "extension library directories".

```shell script
$ cargo ogx run --help
cargo-ogx-run 0.1.0
Heguofeng<hgf199@126.com>
Compile/install extension to a ogx-managed openGauss instance and start gsql

USAGE:
    cargo ogx run [OPTIONS] [ARGS]

ARGS:
    <OG_VERSION>    Do you want to run against openGauss `og3`?
                    [env: OG_VERSION=]
    <DBNAME>        The database to connect to (and create if the first time).  Defaults to a
                    database with the same name as the current extension name

OPTIONS:
        --all-features
            Activate all available features

        --features <FEATURES>
            Space-separated list of features to activate

    -h, --help
            Print help information

        --manifest-path <MANIFEST_PATH>
            Path to Cargo.toml

        --no-default-features
            Do not activate the `default` feature

    -p, --package <PACKAGE>
            Package to build (see `cargo help pkgid`)

        --pgcli
            Use an existing `pgcli` on the $PATH [env: OGX_PGCLI=]

        --profile <PROFILE>
            Specific profile to use (conflicts with `--release`)

    -r, --release
            Compile for release mode (default is debug)

    -v, --verbose
            Enable info logs, -vv for debug, -vvv for trace

    -V, --version
            Print version information
```

## Connect to a Database

```shell script
$ cargo ogx connect
    Re-using existing database strings
psql (13.5)
Type "help" for help.

strings=# select strings.to_lowercase('OGX');
 to_lowercase
--------------
 ogx
(1 row)

strings=# 
```

If you'd simply like to connect to a managed version of openGauss without re-compiling and installing
your extension, use `cargo ogx connect <og3>`.

This command will use the default database named for your extension, or you can specify another
database name as the final argument.

If the specified database doesn't exist, `cargo ogx connect` will create it. Similarly, if
the specified version of openGauss isn't running, it'll be automatically started.

```shell script
$ cargo ogx connect --help
cargo-ogx-connect 0.1.0
Heguofeng<hgf199@126.com>
Connect, via gsql, to a openGauss instance

USAGE:
    cargo ogx connect [OPTIONS] [ARGS]

ARGS:
    <OG_VERSION>    Do you want to run against openGauss `og3`?
                    [env: OG_VERSION=]
    <DBNAME>        The database to connect to (and create if the first time).  Defaults to a
                    database with the same name as the current extension name [env: DBNAME=]

OPTIONS:
    -h, --help
            Print help information

        --manifest-path <MANIFEST_PATH>
            Path to Cargo.toml

    -p, --package <PACKAGE>
            Package to determine default `og_version` with (see `cargo help pkgid`)

        --pgcli
            Use an existing `pgcli` on the $PATH [env: OGX_PGCLI=]

    -v, --verbose
            Enable info logs, -vv for debug, -vvv for trace

    -V, --version
            Print version information
```

## Installing Your Extension Locally

```shell script
$ cargo ogx install
building extension with features ``
"cargo" "build" "--message-format=json-render-diagnostics"
    Finished dev [unoptimized + debuginfo] target(s) in 0.06s

installing extension
     Copying control file to /usr/share/postgresql/13/extension/strings.control
     Copying shared library to /usr/lib/postgresql/13/lib/strings.so
    Building for SQL generation with features ``
    Finished dev [unoptimized + debuginfo] target(s) in 0.06s
 Discovering SQL entities
  Discovered 6 SQL entities: 0 schemas (0 unique), 6 functions, 0 types, 0 enums, 0 sqls, 0 ords, 0 hashes, 0 aggregates
     Writing SQL entities to /usr/share/postgresql/13/extension/strings--0.1.0.sql
    Finished installing strings
```

If for some reason `cargo ogx run <OG_VERSION>` isn't your style, you can use `cargo ogx install` to install your extension
to the openGauss installation described by the `pg_config` tool currently on your `$PATH`.

You'll need write permissions to the directories described by `pg_config --pkglibdir` and `pg_config --sharedir`.

By default, `cargo ogx install` builds your extension in debug mode. Specifying `--release` changes that.

```shell script
$ cargo ogx install --help
cargo-ogx-install 0.1.0
Heguofeng<hgf199@126.com>
Install the extension from the current crate to the openGauss specified by whatever `pg_config` is
currently on your $PATH

USAGE:
    cargo ogx install [OPTIONS]

OPTIONS:
        --all-features
            Activate all available features

    -c, --pg-config <PG_CONFIG>
            The `pg_config` path (default is first in $PATH)

        --features <FEATURES>
            Space-separated list of features to activate

    -h, --help
            Print help information

        --manifest-path <MANIFEST_PATH>
            Path to Cargo.toml

        --no-default-features
            Do not activate the `default` feature

    -p, --package <PACKAGE>
            Package to build (see `cargo help pkgid`)

        --profile <PROFILE>
            Specific profile to use (conflicts with `--release`)

    -r, --release
            Compile for release mode (default is debug)

        --test
            Build in test mode (for `cargo ogx test`)

    -v, --verbose
            Enable info logs, -vv for debug, -vvv for trace

    -V, --version
            Print version information
```

## Testing Your Extension

```shell script
$ cargo ogx test
"cargo" "test" "--features" " og_test"
    Finished test [unoptimized + debuginfo] target(s) in 0.07s
     Running unittests (target/debug/deps/spi-312296af509607bc)

running 2 tests
building extension with features ` og_test`
"cargo" "build" "--features" " og_test" "--message-format=json-render-diagnostics"
    Finished dev [unoptimized + debuginfo] target(s) in 0.06s

installing extension
     Copying control file to /home/ana/.ogx/13.5/ogx-install/share/postgresql/extension/spi.control
     Copying shared library to /home/ana/.ogx/13.5/ogx-install/lib/postgresql/spi.so
    Building for SQL generation with features ` og_test`
    Finished test [unoptimized + debuginfo] target(s) in 0.07s
 Discovering SQL entities
  Discovered 11 SQL entities: 1 schemas (1 unique), 8 functions, 0 types, 0 enums, 2 sqls, 0 ords, 0 hashes, 0 aggregates
     Writing SQL entities to /home/ana/.ogx/13.5/ogx-install/share/postgresql/extension/spi--0.0.0.sql
    Finished installing spi
test tests::og_test_spi_query_by_id_direct ... ok
test tests::og_test_spi_query_by_id_via_spi ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.61s

Stopping openGauss
```

`cargo ogx test [og3]` runs your `#[test]` and `#[og_test]` annotated functions using cargo's test system.

During the testing process, `ogx` starts a tempory instance of openGauss with its `PGDATA` directory in `./target/ogx-test-data-PGVER/`. This openGauss instance is stopped as soon as the test framework has finished.

The output is standard "cargo test" output along with some openGauss log output. In the case of test failures, the failure report will include any openGauss log messages generated by that particular test.

Rust `#[test]` functions behave normally, while `#[og_test]` functions are run **inside** the openGauss instance and have full access to all of openGauss internals. All tests are run in parallel, regardless of their type.

Additionally, a `#[og_test]` function runs in a transaction that is aborted when the test is finished. As such, any changes it might
make to the database are not preserved.

```shell script
$ cargo ogx test --help
cargo-ogx-test 0.1.0
Heguofeng<hgf199@126.com>
Run the test suite for this crate

USAGE:
    cargo ogx test [OPTIONS] [ARGS]

ARGS:
    <OG_VERSION>    Do you want to run against openGauss `og3`,
                    or `all`? [env: OG_VERSION=]
    <TESTNAME>      If specified, only run tests containing this string in their names

OPTIONS:
        --all-features
            Activate all available features

        --features <FEATURES>
            Space-separated list of features to activate

    -h, --help
            Print help information

        --manifest-path <MANIFEST_PATH>
            Path to Cargo.toml

    -n, --no-schema
            Don't regenerate the schema

        --no-default-features
            Do not activate the `default` feature

    -p, --package <PACKAGE>
            Package to build (see `cargo help pkgid`)

        --profile <PROFILE>
            Specific profile to use (conflicts with `--release`)

    -r, --release
            compile for release mode (default is debug)

    -v, --verbose
            Enable info logs, -vv for debug, -vvv for trace

    -V, --version
            Print version information
```

## Building an Installation Package

```shell script
$ cargo ogx package
building extension with features ``
"cargo" "build" "--release" "--message-format=json-render-diagnostics"
    Finished release [optimized] target(s) in 0.07s

installing extension
     Copying control file to target/release/spi-pg13/usr/share/postgresql/13/extension/spi.control
     Copying shared library to target/release/spi-pg13/usr/lib/postgresql/13/lib/spi.so
    Building for SQL generation with features ``
    Finished release [optimized] target(s) in 0.07s
 Discovering SQL entities
  Discovered 8 SQL entities: 0 schemas (0 unique), 6 functions, 0 types, 0 enums, 2 sqls, 0 ords, 0 hashes, 0 aggregates
     Writing SQL entities to target/release/spi-pg13/usr/share/postgresql/13/extension/spi--0.0.0.sql
    Finished installing spi
```

`cargo ogx package [--debug]` builds your extension, in `--release` mode, to a directory structure in
`./target/[debug | release]/extension_name-PGVER` using the openGauss installation path information from the `pg_config`
tool on your `$PATH`.

The intent is that you'd then change into that directory and build a tarball or a .deb or .rpm package.

The directory structure `cargo ogx package` creates starts at the root of the filesystem, as a package-manager installed
version of openGauss is likely to split `pg_config --pkglibdir` and `pg_config --sharedir` into different base paths.

(In the example screenshot above, `cargo ogx package` was used to build a directory structure using my manually installed
version of openGauss 3.1.0.)

This command could be useful from Dockerfiles, for example, to automate building installation packages for various Linux
distobutions or MacOS openGauss installations.

```shell script
$ cargo ogx package --help
cargo-ogx-package 0.1.0
Heguofeng<hgf199@126.com>
Create an installation package directory

USAGE:
    cargo ogx package [OPTIONS]

OPTIONS:
        --all-features
            Activate all available features

    -c, --pg-config <PG_CONFIG>
            The `pg_config` path (default is first in $PATH)

    -d, --debug
            Compile for debug mode (default is release)

        --features <FEATURES>
            Space-separated list of features to activate

    -h, --help
            Print help information

        --manifest-path <MANIFEST_PATH>
            Path to Cargo.toml

        --no-default-features
            Do not activate the `default` feature

        --out-dir <OUT_DIR>
            The directory to output the package (default is `./target/[debug|release]/extname-
            ogXX/`)

    -p, --package <PACKAGE>
            Package to build (see `cargo help pkgid`)

        --profile <PROFILE>
            Specific profile to use (conflicts with `--debug`)

        --test
            Build in test mode (for `cargo ogx test`)

    -v, --verbose
            Enable info logs, -vv for debug, -vvv for trace

    -V, --version
            Print version information
```

## Inspect your Extension Schema

If you just want to look at the full extension schema that ogx will generate, use
`cargo ogx schema`.

```shell script
$ cargo ogx schema --help
cargo-ogx-schema 0.1.0
Heguofeng<hgf199@126.com>
Generate extension schema files

USAGE:
    cargo ogx schema [OPTIONS] [OG_VERSION]

ARGS:
    <OG_VERSION>    Do you want to run against openGauss `og3`?

OPTIONS:
        --all-features
            Activate all available features

    -c, --pg-config <PG_CONFIG>
            The `pg_config` path (default is first in $PATH)

    -d, --dot <DOT>
            A path to output a produced GraphViz DOT file

        --features <FEATURES>
            Space-separated list of features to activate

    -h, --help
            Print help information

        --manifest-path <MANIFEST_PATH>
            Path to Cargo.toml

        --no-default-features
            Do not activate the `default` feature

    -o, --out <OUT>
            A path to output a produced SQL file (default is `stdout`)

    -p, --package <PACKAGE>
            Package to build (see `cargo help pkgid`)

        --profile <PROFILE>
            Specific profile to use (conflicts with `--release`)

    -r, --release
            Compile for release mode (default is debug)

        --skip-build
            Skip building a fresh extension shared object

        --test
            Build in test mode (for `cargo ogx test`)

    -v, --verbose
            Enable info logs, -vv for debug, -vvv for trace

    -V, --version
            Print version information
```

## EXPERIMENTAL: Versioned shared-object support

`ogx` experimentally supports the option to produce a versioned shared library. This allows multiple versions of the
extension to be installed side-by-side, and can enable the deprecation (and removal) of functions between extension
versions. There are some caveats which must be observed when using this functionality. For this reason it is currently
experimental.

### Activation

Versioned shared-object support is enabled by removing the `module_pathname` configuration value in the extension's
`.control` file.

### Concepts

openGauss has the implicit requirement that C extensions maintain ABI compatibility between versions. The idea behind
this feature is to allow interoperability between two versions of an extension when the new version is not ABI
compatible with the old version.

The mechanism of operation is to version the name of the shared library file, and to hard-code function definitions to
point to the versioned shared library file. Without versioned shared-object support, the SQL definition of a C function
would look as follows:

```SQL
CREATE OR REPLACE FUNCTION "hello_extension"() RETURNS text /* &str */
STRICT
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'hello_extension_wrapper';
```

`MODULE_PATHNAME` is replaced by openGauss with the configured value in the `.control` file. For ogx-based extensions,
this is  usually set to `$libdir/<extension-name>`.

When using versioned shared-object support, the same SQL would look as follows:

```SQL
CREATE OR REPLACE FUNCTION "hello_extension"() RETURNS text /* &str */
STRICT
LANGUAGE c /* Rust */
AS '$libdir/extension-0.0.0', 'hello_extension_wrapper';
```

Note that the versioned shared library is hard-coded in the function definition. This corresponds to the
`extension-0.0.0.so` file which `ogx` generates.

It is important to note that the emitted SQL is version-dependent. This means that all previously-defined C functions
must be redefined to point to the current versioned-so in the version upgrade script. As an example, when updating the
extension version to 0.1.0, the shared object will be named `<extension-name>-0.1.0.so`, and `cargo ogx schema` will
produce the following SQL for the above function:

```SQL
CREATE OR REPLACE FUNCTION "hello_extension"() RETURNS text /* &str */
STRICT
LANGUAGE c /* Rust */
AS '$libdir/extension-0.1.0', 'hello_extension_wrapper';
```

This SQL must be used in the upgrade script from `0.0.0` to `0.1.0` in order to point the `hello_extension` function to
the new shared object. `ogx` _does not_ do any magic to determine in which version a function was introduced or modified
and only place it in the corresponding versioned so file. By extension, you can always expect that the shared library
will contain _all_ functions which are still defined in the extension's source code.

This feature is not designed to assist in the backwards compatibility of data types.

### `@MODULE_PATHNAME@` Templating

In case you are already providing custom SQL definitions for Rust functions, you can use the `@MODULE_PATHNAME@`
template in your custom SQL. This value will be replaced with the path to the actual shared object. 

The following example illustrates how this works:

```rust
#[og_extern(sql = r#"
    CREATE OR REPLACE FUNCTION tests."overridden_sql_with_fn_name"() RETURNS void
    STRICT
    LANGUAGE c /* Rust */
    AS '@MODULE_PATHNAME@', '@FUNCTION_NAME@';
"#)]
fn overridden_sql_with_fn_name() -> bool {
    true
}
```

### Caveats

There are some scenarios which are entirely incompatible with this feature, because they rely on some global state in
openGauss, so loading two versions of the shared library will cause trouble.

These scenarios are:
- when using shared memory
- when using query planner hooks
