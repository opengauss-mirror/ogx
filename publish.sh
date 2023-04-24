#! /bin/sh
# Portions Copyright 2019-2021 ZomboDB, LLC.
# Portions Copyright 2021-2022 Technology Concepts & Design, Inc.
# <support@tcdi.com>
#
# All rights reserved.
#
# Use of this source code is governed by the MIT license that can be found in
# the LICENSE file.

DIR=`pwd`
set -x

cd $DIR/ogx-pg-config && cargo publish && sleep 30
cd $DIR/ogx-utils && cargo publish && sleep 30
cd $DIR/ogx-macros && cargo publish && sleep 30
cd $DIR/ogx-pg-sys && cargo publish --no-verify && sleep 30
cd $DIR/ogx && cargo publish --no-verify && sleep 30
cd $DIR/ogx-tests && cargo publish --no-verify && sleep 30
cd $DIR/cargo-ogx && cargo publish # cargo-ogx last so the templates are correct
