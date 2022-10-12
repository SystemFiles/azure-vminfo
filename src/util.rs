use crate::cli::Cli;

use anyhow::Result;

use crate::credentials::AzCredentials;

use lib_vminfo::client::api::RestClient;
use lib_vminfo::models::query::QueryResponse;
use lib_vminfo::models::vm::VirtualMachine;

pub fn config_exists(app_name: &str) -> Result<bool> {
	let path = confy::get_configuration_file_path(app_name, app_name)?;

	if let Ok(metadata) = std::fs::metadata(path) {
		Ok(metadata.is_file())
	} else {
		Ok(false)
	}
}

fn prompt(message: &str, dest: &mut String, sensitive: bool) -> Result<()> {
	use rpassword::prompt_password;
	use std::fmt::write;

	if sensitive {
		let sensitive_var = prompt_password(message)?;

		write(dest, format_args!("{}", sensitive_var))?;
	} else {
		println!("{}", message);

		std::io::stdin().read_line(dest)?;
		dest.pop();
	}
	Ok(())
}

/// Will prompt the user for a set of credentials required to authenticate with Azure Resource Graph
pub fn ask_credentials() -> Result<AzCredentials> {
	let mut tenant_id = String::new();
	let mut client_id = String::new();
	let mut client_secret = String::new();

	prompt("Enter tenant_id: ", &mut tenant_id, false)?;
	prompt("Enter client_id: ", &mut client_id, false)?;
	prompt("Enter client_secret: ", &mut client_secret, true)?;

	Ok(AzCredentials::new(tenant_id, client_id, client_secret))
}

/// Pulls all hosts that match the specified query from lib_vminfo.
pub fn get_vminfo_from_remote(
	client: &RestClient,
	args: &Cli,
) -> anyhow::Result<Vec<VirtualMachine>> {
	let resp: QueryResponse = client.vminfo(
		&args.vm_operand,
		args.match_regexp,
		args.show_extensions,
		None,
		None,
	)?;

	let mut vminfo = resp.data.clone();

	let page_count: u64 = resp.total_results / 1000;
	if page_count > 1 {
		for page in 1..=page_count {
			let skip_count: u64 = page * 1000;
			let rnext = client.vminfo(
				&args.vm_operand,
				args.match_regexp,
				args.show_extensions,
				Some(skip_count),
				None,
			)?;

			vminfo.extend(rnext.data.into_iter());
		}
	}

	Ok(vminfo)
}
