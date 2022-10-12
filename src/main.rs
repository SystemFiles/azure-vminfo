mod cli;
mod util;

use clap::Parser;
use serde::{Deserialize, Serialize};

use cli::Cli;
use util::{ask_credentials, config_exists, get_vminfo_from_remote};

use lib_vminfo::client::api::RestClient;
use lib_vminfo::models::vm::VirtualMachine;

#[derive(Serialize, Deserialize)]
struct Config {
	tenant_id: String,
	client_id: String,
	client_secret: String,
}

impl std::default::Default for Config {
	fn default() -> Self {
		Self {
			tenant_id: String::from("XXXXXX-XXXXX-XXXXXX-XXXXXX"),
			client_id: String::from("XXXXXX-XXXXX-XXXXXX-XXXXXX"),
			client_secret: String::from("XXXXXX-XXXXX-XXXXXX-XXXXXX"),
		}
	}
}

fn main() -> anyhow::Result<()> {
	const APP_NAME: &str = "azure-vminfo";
	let args: Cli = Cli::parse();

	if !config_exists(APP_NAME)? || args.prompt_credentials {
		println!("Azure credentials required ...");
		let mut config: Config = confy::load(APP_NAME, APP_NAME)?;
		let creds = ask_credentials()?;

		// make the changes to running config
		config.tenant_id = creds.tenant_id;
		config.client_id = creds.client_id;
		config.client_secret = creds.client_secret;

		confy::store(APP_NAME, APP_NAME, config)?;

		println!("vminfo configuration updated successfully!");
		return Ok(());
	}

	let config: Config = confy::load(APP_NAME, APP_NAME)?;

	let client = RestClient::new(
		&config.tenant_id.as_str(),
		&config.client_id.as_str(),
		&config.client_secret.as_str(),
		None,
	)?;

	let virtual_machines: Vec<VirtualMachine> = get_vminfo_from_remote(&client, &args)?;

	let result = serde_json::to_string_pretty(&virtual_machines)?;

	println!("{}", result);
	Ok(())
}
