use super::auth;
use super::error;
use crate::models::query::{QueryRequest, QueryResponse};

/// refers to the default endpoint exposed from the Microsoft Resource Graph API
static MANAGEMENT_API_ENDPOINT: &str =
	"https://management.azure.com/providers/Microsoft.ResourceGraph/resources?api-version=2021-03-01";

/// The 'vminfo' client wrapper that helps to abstract away unnecessary implementation details for interacting
/// with the Resource Graph API for the purposes of pulling necessary VM meta and instance data.
#[derive(Debug)]
pub struct RestClient {
	tenant_id: String,
	client_id: String,
	access_token: auth::AccessToken,
	subscription_ids: Option<Vec<String>>,
}

impl std::fmt::Display for RestClient {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{{ tenant_id: {}, client_id: {}, client_secret: *****, access_token: {}, subscription_ids: {:?} }}",
    self.tenant_id, self.client_id, self.access_token.value, self.subscription_ids)
	}
}

impl RestClient {
	/// creates a new instance of the 'vminfo' client
	///
	/// # Arguments
	/// tenant_id: the ID for the target Azure tenant
	/// client_id: the app ID or user principle ID for the identity wishing to authenticate
	/// client_secret: the password to authenticate the identity
	/// subscription_ids: a list of subscriptions to filter to (if `None` specified, will assume all subscriptions in the tenant)
	///
	/// # Example
	///
	/// ```ignore
	/// let client = RestClient::new("tenant_id", "client_id", "client_secret", ["subscription_id1", "subscription_idN"]);
	/// ```
	pub fn new(
		tenant_id: &str,
		client_id: &str,
		client_secret: &str,
		subscription_ids: Option<Vec<String>>,
	) -> error::Result<Self> {
		let access_token = auth::AccessToken::sp_login(tenant_id, client_id, client_secret)?;

		Ok(Self {
			tenant_id: tenant_id.to_string(),
			client_id: client_id.to_string(),
			access_token,
			subscription_ids,
		})
	}

	/// creates a request to pull VM meta and instance data from Azure Resource Graph with filters and extra options possible
	///
	/// # Arguments
	/// query_operand: specifies either a list of full host names for the VM hosts wishing to get data for XOR a single regular expression to match one or more hosts.
	/// 							 if match_regexp = true, will only use the first query_operand for matching
	/// match_regexp: specifies whether to match regular expressions instead of full host names
	/// show_extensions: specifies that vminfo should also return a list of VM extensions that are installed for each host matched
	/// skip: optionally specifies a number of host results to skip to help while working within the constraints of Resource Graph API's paging responses
	/// top: optionally specifies a number of hosts to return for each 'page' (MAXIMUM ALLOWED: 1000)
	///
	/// # Example
	///
	/// ```ignore
	/// let client = RestClient::new("tenant_id", "client_id", "client_secret", ["subscription_id1", "subscription_idN"]);
	/// let resp: Result<QueryResponse> = client.vminfo("l-sykeben-1", false, false, None, None);
	/// ```
	///
	/// ```ignore
	/// let client = RestClient::new("tenant_id", "client_id", "client_secret", ["subscription_id1", "subscription_idN"]);
	/// let resp2: Result<QueryResponse> = client.vminfo("l-.*[0-9]$", true, false, 2000, 1000);
	/// ```
	pub fn vminfo(
		&self,
		query_operand: &Vec<String>,
		match_regexp: bool,
		show_extensions: bool,
		skip: Option<u64>,
		top: Option<u16>,
	) -> error::Result<QueryResponse> {
		let http_client: reqwest::blocking::Client = reqwest::blocking::Client::new();

		if chrono::offset::Utc::now() >= self.access_token.expire_time {
			// access_token expired
			return Err(error::Error::new(
				error::Kind::AccessTokenExpired,
				None::<error::Error>,
				"access token is expired. will not authenticate with resource graph endpoint.",
			));
		}

		let req_body = QueryRequest::make(
			query_operand,
			match_regexp,
			show_extensions,
			skip,
			top,
			&self.subscription_ids,
		);

		let resp: QueryResponse = http_client
			.post(MANAGEMENT_API_ENDPOINT)
			.bearer_auth(&self.access_token)
			.json(&req_body)
			.send()
			.map_err(|err| error::vm_request(err, "HTTP request to Resource Graph failed"))?
			.json()
			.map_err(|err| {
				error::vm_request(
					err,
					"HTTP response data from Resource Graph cannot be mapped to struct",
				)
			})?;

		if resp.data.len() == 0 {
			return Err(error::Error::new(
				error::Kind::VMNotFound,
				None::<error::Error>,
				"no vm results were returned for the specified query",
			));
		}

		Ok(resp)
	}
}
