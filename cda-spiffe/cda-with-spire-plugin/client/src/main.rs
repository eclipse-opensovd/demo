// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Contributors to the Eclipse Foundation
// Assisted By: Codex (GPT-6 Sol High)

use clap::Parser;
use spiffe::WorkloadApiClient;

#[derive(Parser)]
#[command(about = "Call CDA using a fresh SPIRE JWT-SVID")]
struct Args {
    #[arg(
        long,
        default_value = "http://cda:20002/vehicle/v15/components/blueprint-ecu/data/powertrain_mode"
    )]
    url: String,
    #[arg(long, default_value = "sovd.cda")]
    audience: String,
    #[arg(long)]
    without_token: bool,
    #[arg(long, default_value_t = 200)]
    expect_status: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let client = reqwest::Client::new();
    let mut request = client.get(&args.url);
    if !args.without_token {
        let workload_api = WorkloadApiClient::connect_env().await?;
        let svid = workload_api
            .fetch_jwt_svid(&[args.audience.as_str()], None)
            .await?;
        println!("Caller identity: {}", svid.spiffe_id());
        request = request.bearer_auth(svid.token());
    }

    let response = request.send().await?;
    let status = response.status();
    println!("HTTP {status}");
    println!("{}", response.text().await?);
    if status.as_u16() != args.expect_status {
        return Err(format!("expected HTTP {}, got {status}", args.expect_status).into());
    }
    Ok(())
}
