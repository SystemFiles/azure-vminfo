mod cli;
mod credentials;
mod util;

use std::process;

use clap::Parser;
use lib_vminfo::error::auth;
use lib_vminfo::vm::VirtualMachine;
use lib_vminfo::{auth::Method, error::AuthErrorKind};
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
	if args.perform_login {
		if args.use_service_principal {
			let creds = ask_credentials(Method::ClientCredentials)?;
			let _ = LocalClient::new(
				APP_NAME,
				&creds.tenant_id,
				&creds.client_id,
				creds.client_secret,
				None,
			)?
			.login_client_credentials(true)?;
		} else {
			let creds = ask_credentials(Method::DeviceCode)?;
			let _ = LocalClient::new(
				APP_NAME,
				&creds.tenant_id,
				&creds.client_id,
				creds.client_secret,
				None,
			)?
			.login_device_code(true)?;
		}
		println!("login successful!");

		process::exit(0)
	} else if args.perform_logout {
		println!("clearing stored credentials");
		LocalClient::new(APP_NAME, "", "", None, None)?.clear_credential_cache()?;
		println!("stored credentials have been removed and client has been deauthenticated");

		process::exit(0)
	}

	client = match LocalClient::from_store(APP_NAME) {
		Ok(c) => c,
		Err(_) => {
			return Err(auth(
				None::<lib_vminfo::error::Error>,
				AuthErrorKind::MissingToken,
				"missing credentials for client. re-run with '--login' to authenticate",
			))?
		}
	};

	let virtual_machines: Vec<VirtualMachine> = get_vminfo_from_remote(&client, &args)?;

	let result = serde_json::to_string_pretty(&virtual_machines)?;

	println!("{}", result);
	Ok(())
}
