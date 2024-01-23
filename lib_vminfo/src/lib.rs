#![deny(missing_docs)]
//!
//! A small library designed to make querying detailed VM information from Azure Resource Graph as simple and painless as possible
//!
//! ## Installation
//!
//! To install and use this library, simply add it to your `[dependencies]` in your `Cargo.toml`
//!
//! ## Getting Started
//!
//! The first thing to setup is your App Registration which will be used either as a client directly or through user-impersonation from an AAD user.
//!
//! ### Create the App Registration
//!
//! Create an App Registration with a name of your choosing and ensure that the Enterprise App is able to read all on
//! the tenant.
//!
//! then ensure the following API permissions are set:
//!
//! - Azure Service Management > user_impersonation
//! - Migrosoft Graph > User.Read
//!
//! Then make sure an admin provides consent for the Directory which contains your AAD users. Also make sure that
//! any users that should be able to impersonate the Enterprise App are added as owners in the App Registration
//!
//! Under the `Authentication` section, add a redirect URI: `https://global.consent.azure-apim.net/redirect`
//! then ensure that you check the boxes to allow Access tokens to be issued by the authroization endpoint
//!
//! Finally, under Advanced settings in the `Authentication` section, switch `Allow public client flows` to "Yes".
//!
//! ### Decide on an Authentication Method
//!
//! - Client Credentials (uses the Service Account (Enterprise App) directly)
//! - User Impersonation (uses a user account to impersonate the Service Account (Enterprise App))
//!
//! ### Client Credentials
//!
//! Create a Secret in the `Certificates and Secrets` section of the App Registration.
//! Record the Secret value as well as tenant ID and Client(app) ID for later.
//!
//! ### User Impersonation
//!
//! Record the Tenant ID and Client(app) ID for the App Registration from the `Overview Section` for later.
//!
//! ```ignore
//! [dependencies]
//! lib_vminfo = { version = "1.0", path = "./lib_vminfo" }
//! ```
//!
//! ## Basic Usage
//!
//! Below is basic usage of the VMInfo Client to grab VMs matching a regular expression and caching credentials locally in a file.
//!
//! ```ignore
//!
//! // get the first 100 VMs that match the provided regexp
//! let resp: QueryResponse = client.query_vminfo(
//!		vec!["ubuntu-vm[0-9]+"],
//!		true,
//!		false,
//!		Some(0),
//!		Some(100),
//!	)?;
//!
//! ...
//! ```
//!
//! ## Maintainer(s)
//!
//! - Ben Sykes (ben.sykes@statcan.gc.ca)
//!
//!

///
/// defines authentication behaviour and data types for handling Azure authentication
///
pub mod auth;
///
/// defines error and result types used in the client library
///
pub mod error;

///
/// defines data structures for caching API responses for various requests
///
pub mod caching;

///
/// defines types for handling persistence of authentication details (tokens / client credentials)
///
pub mod persistance;
///
/// Query Request and Response types
///
pub mod query;
///
/// Virtual Machine Response Types
///
pub mod vm;

use std::fmt::{Debug, Display};

use caching::redis_cache::VMResultsCacheRedis;
use caching::Cache;

use crate::query::QueryResponseType;
use crate::query::{QueryRequest, QueryResponse};
use auth::{AzCredentials, Method};
use error::{AuthErrorKind, Error, Kind, VMInfoResult};
use persistance::{FileTokenStore, PersistantStorage};
use serde::{Deserialize, Serialize};
use vm::VirtualMachine;

///
/// default management endpoint for querying data from Resource Graph
///
const MANAGEMENT_API_ENDPOINT: &str =
	"https://management.azure.com/providers/Microsoft.ResourceGraph/resources?api-version=2021-03-01";

///
/// Defines AuthTokens as a pair of access and refresh tokens
///
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AuthTokens {
	///
	/// defines an oauth2.0 access token
	///
	pub access_token: String,
	///
	/// defines an oauth2.0 refresh token
	///
	#[serde(skip_serializing_if = "Option::is_none")]
	pub refresh_token: Option<String>,
}

impl Default for AuthTokens {
	fn default() -> Self {
		AuthTokens {
			access_token: "XXX".to_string(),
			refresh_token: None,
		}
	}
}

///
/// Defines the vminfo Client (with caching)
///
#[derive(Debug, Clone)]
pub struct Client<PS, RC>
where
	PS: PersistantStorage<AzCredentials>,
	RC: Cache<VirtualMachine> + Clone,
{
	tenant_id: String,
	client_id: String,
	client_secret: Option<String>,
	token_store: PS,
	result_cache: Option<RC>,
	subscriptions: Option<Vec<String>>,
}

///
/// implementation of specific client methods that rely on a Local File Credential Store and Redis Result Cache
///
/// this client provides flexibility for alternative implementations with a custom selection of Token/Credential persistance
/// and VM Results cache implementations.
///
/// Bounds only on PersistantStorage that is capable of persisting 'AzCredentials' and Cache capable of caching VirtualMachine results
///
impl Client<FileTokenStore, VMResultsCacheRedis> {
	///
	/// creates a new Client using the 'FileTokenStore' persistence method and 'VMResultsCacheRedis' cache
	///
	pub fn new(
		app_name: &str,
		tenant_id: &str,
		client_id: &str,
		client_secret: Option<String>,
		redis_host: Option<&str>,
		redis_port: Option<u16>,
		redis_password: Option<String>,
		redis_use_tls: Option<bool>,
		subscriptions: Option<Vec<String>>,
	) -> VMInfoResult<Self> {
		Ok(Self {
			tenant_id: String::from(tenant_id),
			client_id: String::from(client_id),
			client_secret,
			token_store: FileTokenStore::new(app_name)?,
			result_cache: match redis_host {
				Some(h) => Some(VMResultsCacheRedis::new(
					h,
					redis_port.unwrap_or(6379u16),
					redis_password,
					redis_use_tls.unwrap_or(false),
				)?),
				_ => None,
			},
			subscriptions,
		})
	}

	///
	/// creates a new vminfo Client from a persistant storage method
	///
	pub fn from_store(
		app_name: &str,
		redis_host: Option<&str>,
		redis_port: Option<u16>,
		redis_password: Option<String>,
		redis_use_tls: Option<bool>,
	) -> VMInfoResult<Self> {
		let mut c = Self {
			tenant_id: "".to_string(),
			client_id: "".to_string(),
			client_secret: None,
			token_store: FileTokenStore::new(app_name)?,
			result_cache: match redis_host {
				Some(h) => Some(VMResultsCacheRedis::new(
					h,
					redis_port.unwrap_or(6739u16),
					redis_password,
					redis_use_tls.unwrap_or(false),
				)?),
				_ => None,
			},
			subscriptions: None,
		};

		c.load_credentials()
	}
}

impl<PS, RC> Client<PS, RC>
where
	PS: PersistantStorage<AzCredentials>,
	RC: Cache<VirtualMachine> + Clone,
{
	///
	/// performs login with Azure authentication server using the client_credentials OAuth2.0 flow described by [RFC6749](https://www.rfc-editor.org/rfc/rfc6749#section-4.4)
	///
	pub fn login_client_credentials(mut self, force: bool) -> VMInfoResult<Self> {
		let _ = self.load_credentials();

		match self.access_token() {
			Some(_) => {
				if !force {
					Ok(self)
				} else {
					let tokens = auth::login_non_interactive(&auth::Configuration::new(
						&self.tenant_id.as_str(),
						&self.client_id.as_str(),
						&self.client_secret,
					))?;

					self.save_credentials(&tokens)?;

					Ok(self)
				}
			}
			_ => {
				let tokens = auth::login_non_interactive(&auth::Configuration::new(
					&self.tenant_id.as_str(),
					&self.client_id.as_str(),
					&self.client_secret,
				))?;

				self.save_credentials(&tokens)?;

				Ok(self)
			}
		}
	}
	///
	/// performs login with Azure authentication server using the devicecode OAuth2.0 flow described by [RFC8628](https://www.rfc-editor.org/rfc/rfc8628#section-3.4)
	///
	pub fn login_device_code(mut self, force: bool) -> VMInfoResult<Self> {
		let _ = self.load_credentials();

		match self.access_token() {
			Some(_) => {
				if !force {
					Ok(self)
				} else {
					let tokens = auth::login_interactive(&auth::Configuration::new(
						&self.tenant_id.as_str(),
						&self.client_id.as_str(),
						&None,
					))?;

					self.save_credentials(&tokens)?;

					Ok(self)
				}
			}
			_ => {
				let tokens = auth::login_interactive(&auth::Configuration::new(
					&self.tenant_id.as_str(),
					&self.client_id.as_str(),
					&None,
				))?;

				self.save_credentials(&tokens)?;

				Ok(self)
			}
		}
	}

	fn reauth(&self) -> VMInfoResult<Self> {
		match self.auth_method() {
			Method::ClientCredentials => self.clone().login_client_credentials(true),
			Method::DeviceCode => self.clone().login_device_code(true),
		}
	}

	///
	/// determines which authentication method is being used as primary on the client
	///
	pub fn auth_method(&self) -> Method {
		match self.client_secret {
			Some(_) => Method::ClientCredentials,
			None => Method::DeviceCode,
		}
	}

	///
	/// saves any active auth credentials from the client into persistant storage.
	///
	/// ## Possible Failures
	///
	/// - Will return an error if there are no active tokens in the client (must authenticate first using login()).
	/// - This function may also fail if there is a problem writing to the persistant storage.
	///
	pub fn save_credentials(&self, auth_tokens: &AuthTokens) -> VMInfoResult<()> {
		// if let Some(access_token) = self.access_token() {
		// 	let client_credentials = AzCredentials {
		// 		tenant_id: self.tenant_id.clone(),
		// 		client_id: self.client_id.clone(),
		// 		client_secret: self.client_secret.clone(),
		// 		tokens: AuthTokens {
		// 			access_token,
		// 			refresh_token: self.refresh_token(),
		// 		},
		// 	};

		let client_credentials = AzCredentials {
			tenant_id: self.tenant_id.clone(),
			client_id: self.client_id.clone(),
			client_secret: self.client_secret.clone(),
			tokens: auth_tokens.clone(),
		};

		self.token_store.write(&client_credentials)?;

		Ok(())
	}

	///
	/// public vminfo query request method that wraps request() with special authentication handlers
	///
	/// ## authentication errors
	///
	/// This function will handle regular use authentication errors for you in processing your request,
	/// but will throw an error back to the client if the authentication error cannot be resolved automatically.
	/// This includes, but is no limited to:
	///
	/// - Bad Credentials
	/// - Invalid request
	/// - Network timeouts
	/// - failed token refresh
	/// - Permissions errors on scope or otherwise
	///
	pub fn query_vminfo(
		&self,
		query_operand: &Vec<String>,
		match_regexp: bool,
		show_extensions: bool,
		show_tags: bool,
		nocache: bool,
		skip: Option<u64>,
		top: Option<u16>,
	) -> VMInfoResult<QueryResponse> {
		let mut query_ops: Vec<String> = query_operand.clone();
		let mut cached_results: Vec<VirtualMachine> = Vec::new();

		if !nocache {
			match self.clone().result_cache {
				Some(cache) => {
					query_ops = Vec::new();
					for (_, q) in query_operand.into_iter().enumerate() {
						match cache.get(q.to_lowercase().as_str()) {
							Ok(vm) => {
								cached_results.push(vm);
							}
							_ => query_ops.push(q.clone()),
						}
					}
				}
				_ => (),
			};
		}

		if query_ops.len() > 0 {
			let resp: VMInfoResult<QueryResponse> =
				self.request(
					&query_ops,
					match_regexp,
					show_extensions,
					show_tags,
					skip,
					top,
				);

			match resp {
				Ok(mut r) => {
					r.data.append(&mut cached_results);
					Ok(r)
				}
				Err(err) => match err.kind() {
					Kind::AuthenticationError(aek) => match aek {
						AuthErrorKind::MissingToken => {
							self
								.reauth()?
								.request(
									&query_ops,
									match_regexp,
									show_extensions,
									show_tags,
									skip,
									top,
								)
						}
						AuthErrorKind::TokenExpired => match self.auth_method() {
							Method::ClientCredentials => {
								self
									.reauth()?
									.request(
										&query_ops,
										match_regexp,
										show_extensions,
										show_tags,
										skip,
										top,
									)
							}
							Method::DeviceCode => self.clone().exchange_refresh_token()?.request(
								&query_ops,
								match_regexp,
								show_extensions,
								show_tags,
								skip,
								top,
							),
						},
						_ => Err(err)?,
					},
					Kind::NoneFoundError => {
						if cached_results.len() > 0 {
							Ok(QueryResponse {
								total_results: cached_results.len() as u64,
								data: cached_results,
							})
						} else {
							Err(err)?
						}
					}
					_ => Err(err)?,
				},
			}
		} else {
			Ok(QueryResponse {
				total_results: cached_results.len() as u64,
				data: cached_results,
			})
		}
	}

	/// creates a request to pull VM meta and instance data from Azure Resource Graph with filters and extra options possible
	///
	/// ## Arguments
	/// - query_operand: specifies either a list of full host names for the VM hosts wishing to get data for XOR a single regular expression to match one or more hosts.
	/// 							 	 if match_regexp = true, will only use the first query_operand for matching
	/// - match_regexp: specifies whether to match regular expressions instead of full host names
	/// - show_extensions: specifies that vminfo should also return a list of VM extensions that are installed for each host matched
	/// - skip: optionally specifies a number of host results to skip to help while working within the constraints of Resource Graph API's paging responses
	/// - top: optionally specifies a number of hosts to return for each 'page' (MAXIMUM ALLOWED: 1000)
	fn request(
		&self,
		query_operand: &Vec<String>,
		match_regexp: bool,
		show_extensions: bool,
		show_tags: bool,
		skip: Option<u64>,
		top: Option<u16>,
	) -> VMInfoResult<QueryResponse> {
		let http_client: reqwest::blocking::Client = reqwest::blocking::Client::new();

		let req_body = QueryRequest::make(
			query_operand,
			match_regexp,
			show_extensions,
			show_tags,
			skip,
			top,
			&self.subscriptions,
		);

		let access_token_opt = match self.access_token() {
			Some(t) => t,
			_ => Err(error::auth(
				None::<Error>,
				AuthErrorKind::MissingToken,
				"no access token provided for request",
			))?,
		};

		let resp: QueryResponseType = http_client
			.post(MANAGEMENT_API_ENDPOINT)
			.bearer_auth(&access_token_opt)
			.json(&req_body)
			.send()
			.map_err(|err| {
				let status = err.status();
				error::request(
					Some(err),
					status,
					"request for vm info from Resource Graph failed",
				)
			})?
			.json()
			.map_err(|err| {
				let status = err.status();
				error::request(
					Some(err),
					status,
					"could not parse vm info into valid response object",
				)
			})?;

		match resp {
			QueryResponseType::Ok(r) => {
				if r.data.len() == 0 {
					return Err(error::none_found(
						None::<error::Error>,
						format!(
							"no virtual machines were found with the provided query: {:?}",
							query_operand
						)
						.as_str(),
					));
				}

				match self.clone().result_cache {
					Some(cache) => {
						for (_, vm) in r.clone().data.into_iter().enumerate() {
							cache.put(vm.clone().vm_name.unwrap().to_lowercase().as_str(), &vm)?;
						}
					}
					_ => (),
				};

				Ok(r)
			}
			QueryResponseType::Err { error } => {
				return Err(error::auth(
					None::<Error>,
					if error.code == "ExpiredAuthenticationToken".to_string() {
						AuthErrorKind::TokenExpired
					} else if error.code == "InvalidAuthenticationToken".to_string() {
						AuthErrorKind::BadCredentials
					} else if error.code == "AccessDenied".to_string() {
						AuthErrorKind::AccessDenied
					} else {
						AuthErrorKind::BadRequest
					},
					format!("{}: {}", error.code, error.message).as_str(),
				))?;
			}
		}
	}

	///
	/// retrieves any stored credentials from persistant storage and writes them to the clients memory
	///
	/// ## Possible Failures
	///
	/// - May fail to read persistant storage
	///
	pub fn load_credentials(&self) -> VMInfoResult<AzCredentials> {
		let client_credentials = self.token_store.read()?;

		Ok(client_credentials)
	}

	///
	/// will exchange a refresh token using the auth module for a new set of access and refresh tokens
	///
	pub fn exchange_refresh_token(&self) -> VMInfoResult<AuthTokens> {
		let client_credentials = self.load_credentials()?;
		let tokens: AuthTokens = auth::exchange_refresh_tokens(&self.tenant_id, &self.client_id, client_credentials.tokens.refresh_token)?;

		self.save_credentials(&tokens)?;

		Ok(tokens)
	}
	///
	/// clears credentials from token/credential cache
	///
	pub fn clear_credential_cache(&self) -> VMInfoResult<()> {
		self.token_store.clear()
	}
}

impl<PS, RC> AsMut<Client<PS, RC>> for Client<PS, RC>
where
	PS: PersistantStorage<AzCredentials>,
	RC: Cache<VirtualMachine> + Clone,
{
	fn as_mut(&mut self) -> &mut Client<PS, RC> {
		self
	}
}

impl<PS, RC> Display for Client<PS, RC>
where
	PS: PersistantStorage<AzCredentials>,
	RC: Cache<VirtualMachine> + Clone,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"VMInfo Client {{ tenant_id: {}, client_id: {}, client_secret: [redacted], active_tokens: [redacted] }}",
			self.tenant_id, self.client_id
		)
	}
}

///
/// defines a Client which uses local disk storage to persist credential/token data for vminfo
///
pub type LocalClient = Client<FileTokenStore, VMResultsCacheRedis>;
