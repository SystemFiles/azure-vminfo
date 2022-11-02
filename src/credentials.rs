#[derive(Debug, Clone)]
pub struct CliCredentials {
	pub tenant_id: String,
	pub client_id: String,
	pub client_secret: Option<String>,
}

impl CliCredentials {
	pub fn new(tenant_id: String, client_id: String, client_secret: Option<String>) -> Self {
		Self {
			tenant_id,
			client_id,
			client_secret,
		}
	}
}

impl std::fmt::Display for CliCredentials {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{{ tenant_id: {}, client_id: {}, client_secret: {} }}",
			self.tenant_id,
			self.client_id,
			"X".repeat(self.client_secret.clone().unwrap_or("".to_string()).len())
		)
	}
}
