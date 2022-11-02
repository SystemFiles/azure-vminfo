use crate::cli::Cli;
use crate::credentials::CliCredentials;

use anyhow::Result;

use lib_vminfo::auth::Method;
use lib_vminfo::LocalClient;

use lib_vminfo::query::QueryResponse;
use lib_vminfo::vm::VirtualMachine;

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

///
/// Will prompt the user for a set of credentials required to authenticate with Azure Resource Graph
///
pub fn ask_credentials(method: Method) -> Result<CliCredentials> {
	let mut tenant_id = String::new();
	let mut client_id = String::new();
	let mut client_secret = String::new();

	prompt("Enter tenant_id: ", &mut tenant_id, false)?;
	prompt("Enter client_id: ", &mut client_id, false)?;

	if method == Method::ClientCredentials {
		prompt("Enter client_secret/password: ", &mut client_secret, true)?;
	}

	let client_secret_opt: Option<String> = if client_secret.len() > 0 {
		Some(client_secret)
	} else {
		None
	};

	Ok(CliCredentials::new(tenant_id, client_id, client_secret_opt))
}

///
/// Pulls all hosts that match the specified query from lib_vminfo.
///
pub fn get_vminfo_from_remote(
	client: &LocalClient,
	args: &Cli,
) -> anyhow::Result<Vec<VirtualMachine>> {
	let resp: QueryResponse = client.query_vminfo(
		&args.vm_operand,
		args.match_regexp,
		args.show_extensions,
		None,
		None,
	)?;

	let mut vminfo: Vec<VirtualMachine> = resp.data.clone();

	let page_count: u64 = resp.total_results / 1000;
	if page_count > 1 {
		for page in 1..=page_count {
			let skip_count: u64 = page * 1000;
			let rnext: QueryResponse = client.query_vminfo(
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
