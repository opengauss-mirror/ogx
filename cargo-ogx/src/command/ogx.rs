/*
Portions Copyright 2019-2021 ZomboDB, LLC.
Portions Copyright 2021-2022 Technology Concepts & Design, Inc. <support@tcdi.com>

All rights reserved.

Use of this source code is governed by the MIT license that can be found in the LICENSE file.
*/
use crate::CommandExecute;

#[derive(clap::Args, Debug)]
#[clap(about, author)]
pub(crate) struct Ogx {
    #[clap(subcommand)]
    subcommand: CargoOgxSubCommands,
    #[clap(from_global, parse(from_occurrences))]
    verbose: usize,
}

impl CommandExecute for Ogx {
    fn execute(self) -> eyre::Result<()> {
        self.subcommand.execute()
    }
}

#[derive(clap::Subcommand, Debug)]
enum CargoOgxSubCommands {
    Init(super::init::Init),
    Start(super::start::Start),
    Stop(super::stop::Stop),
    Status(super::status::Status),
    New(super::new::New),
    Install(super::install::Install),
    Package(super::package::Package),
    Schema(super::schema::Schema),
    Run(super::run::Run),
    Connect(super::connect::Connect),
    Test(super::test::Test),
    Get(super::get::Get),
}

impl CommandExecute for CargoOgxSubCommands {
    fn execute(self) -> eyre::Result<()> {
        use CargoOgxSubCommands::*;
        match self {
            Init(c) => c.execute(),
            Start(c) => c.execute(),
            Stop(c) => c.execute(),
            Status(c) => c.execute(),
            New(c) => c.execute(),
            Install(c) => c.execute(),
            Package(c) => c.execute(),
            Schema(c) => c.execute(),
            Run(c) => c.execute(),
            Connect(c) => c.execute(),
            Test(c) => c.execute(),
            Get(c) => c.execute(),
        }
    }
}
