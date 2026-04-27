#!/bin/bash
# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: 2025 The Contributors to Eclipse OpenSOVD (see CONTRIBUTORS)
#
# See the NOTICE file(s) distributed with this work for additional
# information regarding copyright ownership.
#
# This program and the accompanying materials are made available under the
# terms of the Apache License Version 2.0 which is available at
# https://www.apache.org/licenses/LICENSE-2.0

source $HOME/google_secrets

ps aux | grep sim | grep -v grep | awk '{print $2}' | xargs kill -9 >/dev/null

pushd ../ecu-sim
./gradlew run &
sleep 5
popd

export RUST_LOG=trace
address="$(ifconfig en0 inet | awk '/inet / {print $2}')"
cargo run -- --tester-address $address --gateway-port 13400 --databases-path /Users/MOHALEX/dev/ocx-demo-2026/ecu-databases/
