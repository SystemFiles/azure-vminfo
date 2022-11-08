mod cli;
mod credentials;
mod util;

use std::process;

use clap::Parser;
use lib_vminfo::error::auth;
use lib_vminfo::vm::VirtualMachine;
use lib_vminfo::{auth::Method, error::AuthErrorKind};

use cli::Cli;
use lib_vminfo::LocalClient;
use serde::{Deserialize, Serialize};
use util::get_vminfo_from_remote;

use crate::util::ask_credentials;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppConfig {
	use_cache: bool,
	redis_host: String,
	redis_port: u16,
	redis_password: Option<String>,
	redis_use_tls: bool,
	subscriptions: Option<Vec<String>>,
	log_level: String,
}

impl Default for AppConfig {
	fn default() -> Self {
		Self {
			use_cache: true,
			redis_host: "127.0.0.1".to_string(),
			redis_port: 6379u16,
			redis_password: None,
			redis_use_tls: false,
			subscriptions: None,
			log_level: "INFO".to_string(),
		}
	}
}

fn main() -> anyhow::Result<()> {
	const APP_NAME: &str = "azure-vminfo";
	let config: AppConfig = confy::load(APP_NAME, "config")?;
	let args: Cli = Cli::parse();

	let client: LocalClient;
	if args.perform_login {
		if args.use_service_principal {
			let creds = ask_credentials(Method::ClientCredentials)?;
			if config.use_cache {
				let _ = LocalClient::new(
					APP_NAME,
					&creds.tenant_id,
					&creds.client_id,
					creds.client_secret,
					Some(config.redis_host.as_str()),
					Some(config.redis_port),
					config.redis_password,
					Some(config.redis_use_tls),
					None,
				)?
				.login_client_credentials(true)?;
			} else {
				let _ = LocalClient::new(
					APP_NAME,
					&creds.tenant_id,
					&creds.client_id,
					creds.client_secret,
					None,
					None,
					None,
					None,
					None,
				)?
				.login_client_credentials(true)?;
			}
		} else {
			let creds = ask_credentials(Method::DeviceCode)?;
			if config.use_cache {
				let _ = LocalClient::new(
					APP_NAME,
					&creds.tenant_id,
					&creds.client_id,
					creds.client_secret,
					Some(config.redis_host.as_str()),
					Some(config.redis_port),
					config.redis_password,
					Some(config.redis_use_tls),
					None,
				)?
				.login_device_code(true)?;
			} else {
				let _ = LocalClient::new(
					APP_NAME,
					&creds.tenant_id,
					&creds.client_id,
					creds.client_secret,
					None,
					None,
					None,
					None,
					None,
				)?
				.login_device_code(true)?;
			}
		}
		println!("login successful!");

		process::exit(0)
	} else if args.perform_logout {
		println!("clearing stored credentials");
		LocalClient::new(APP_NAME, "", "", None, None, None, None, None, None)?
			.clear_credential_cache()?;
		println!("stored credentials have been removed and client has been deauthenticated");

		process::exit(0)
	}

	if config.use_cache {
		client = match LocalClient::from_store(
			APP_NAME,
			Some(config.redis_host.as_str()),
			Some(config.redis_port),
			config.redis_password,
			Some(config.redis_use_tls),
		) {
			Ok(c) => c,
			Err(_) => {
				return Err(auth(
					None::<lib_vminfo::error::Error>,
					AuthErrorKind::MissingToken,
					"missing credentials for client. re-run with '--login' to authenticate",
				))?
			}
		}
	} else {
		client = match LocalClient::from_store(APP_NAME, None, None, None, None) {
			Ok(c) => c,
			Err(_) => {
				return Err(auth(
					None::<lib_vminfo::error::Error>,
					AuthErrorKind::MissingToken,
					"missing credentials for client. re-run with '--login' to authenticate",
				))?
			}
		}
	}

	let virtual_machines: Vec<VirtualMachine> = get_vminfo_from_remote(&client, &args)?;
	let result = serde_json::to_string_pretty(&virtual_machines)?;

	println!("{}", result);
	Ok(())
}
