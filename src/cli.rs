use clap::Parser;

/// A Rust utility to pull detailed virtual machine data from a configured Azure tenant using the Azure Resource Graph APIs
#[derive(Debug, Parser)]
pub struct Cli {
	/// Specifies one or more VM name(s) or a regular expression to match VM(s)
	#[arg(value_name = "vm_name_or_regexp")]
	#[arg(required_unless_present("perform_login"))]
	#[arg(required_unless_present("perform_logout"))]
	pub vm_operand: Vec<String>,

	/// Specifies whether to prompt for credentials manually (will exit). Will default to user authentication method.
	#[arg(long = "login", required = false)]
	pub perform_login: bool,

	/// Perform full logout operation. This will clear the credential/token cache and remove the user from the system
	#[arg(long = "logout", required = false)]
	pub perform_logout: bool,

	/// Specifies that azure-vminfo should use a service-principal (client_id and client_secret) to authenticate
	#[arg(long = "service-principal", required = false)]
	pub use_service_principal: bool,

	/// Specifies that azure-vminfo should use an interactive (client_id and login challenge) authentication method
	#[arg(long = "interactive", required = false)]
	pub interactive_login: bool,

	/// Specifies whether to ignore the cache and force data to be pulled from Resource Graph API directly
	#[arg(short = 'c', long = "no-cache", required = false)]
	pub no_cache: bool,

	/// Specifies whether or not to enable regexp matching
	#[arg(short = 'r', long = "match-regexp", required = false)]
	pub match_regexp: bool,

	/// Specifies whether or not to display Azure extensions for each VM
	#[arg(short = 'e', long = "extensions", required = false)]
	pub show_extensions: bool,
}

impl std::fmt::Display for Cli {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"vminfo: Name(s): {:?}, Regex: {}, Show Extensions: {}",
			self.vm_operand, self.match_regexp, self.show_extensions
		)
	}
}

impl Default for Cli {
	fn default() -> Self {
		Self {
			vm_operand: vec!["".to_string()],
			match_regexp: false,
			show_extensions: false,
			perform_login: false,
			perform_logout: false,
			no_cache: false,
			use_service_principal: false,
			interactive_login: true,
		}
	}
}
