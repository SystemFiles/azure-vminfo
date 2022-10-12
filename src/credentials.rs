#[derive(Debug, Clone)]
pub struct AzCredentials {
	pub tenant_id: String,
	pub client_id: String,
	pub client_secret: String,
}

impl AzCredentials {
	pub fn new(tenant_id: String, client_id: String, client_secret: String) -> Self {
		Self {
			tenant_id,
			client_id,
			client_secret,
		}
	}
}

impl std::fmt::Display for AzCredentials {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{{ tenant_id: {}, client_id: {}, client_secret: {} }}",
			self.tenant_id,
			self.client_id,
			"X".repeat(self.client_secret.len())
		)
	}
}
