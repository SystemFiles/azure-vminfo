mod cli;
mod credentials;
mod util;

use std::process;

use clap::Parser;
use lib_vminfo::auth::Method;
use lib_vminfo::error::auth;
use lib_vminfo::vm::VirtualMachine;
use serde::{Deserialize, Serialize};

use cli::Cli;
use lib_vminfo::LocalClient;
use util::get_vminfo_from_remote;

use crate::util::ask_credentials;

#[derive(Serialize, Deserialize)]
struct Config {
	tenant_id: String,
	client_id: String,
	client_secret: Option<String>,
}

impl std::default::Default for Config {
	fn default() -> Self {
		Self {
			tenant_id: String::from("XXXXXX-XXXXX-XXXXXX-XXXXXX"),
			client_id: String::from("XXXXXX-XXXXX-XXXXXX-XXXXXX"),
			client_secret: None,
		}
	}
}

fn main() -> anyhow::Result<()> {
	const APP_NAME: &str = "azure-vminfo";
	let args: Cli = Cli::parse();

	let client: LocalClient;
	if args.prompt_credentials {
		if args.use_service_principal {
			let creds = ask_credentials(Method::ClientCredentials)?;
			let _ = LocalClient::new(
				APP_NAME,
				&creds.tenant_id,
				&creds.client_id,
				creds.client_secret,
				None,
			)?
			.login(Method::ClientCredentials)?;
		} else {
			let creds = ask_credentials(Method::DeviceCode)?;
			let _ = LocalClient::new(
				APP_NAME,
				&creds.tenant_id,
				&creds.client_id,
				creds.client_secret,
				None,
			)?
			.login(Method::DeviceCode)?;
		}
		println!("login successful!");
		process::exit(0)
	}

	client = match LocalClient::from_store(APP_NAME) {
		Ok(c) => c,
		Err(e) => {
			return Err(auth(
				Some(e),
				lib_vminfo::error::AuthErrorKind::BadCredentials,
				"missing credentials for client. re-run with '--login' to authenticate",
			))?
		}
	};

	let virtual_machines: Vec<VirtualMachine> =
		get_vminfo_from_remote(&client, &args, &client.auth_method())?;

	let result = serde_json::to_string_pretty(&virtual_machines)?;

	println!("{}", result);
	Ok(())
}
