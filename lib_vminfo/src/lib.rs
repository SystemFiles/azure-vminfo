#![warn(missing_docs)]
//!
//! A small library designed to make querying detailed VM information from Azure Resource Graph as simple and painless as possible
//!
//! ## Installation
//!
//! To install and use this library, simply add it to your `[dependencies]` in your `Cargo.toml`
//!
//! ```ignore
//! [dependencies]
//! lib_vminfo = { version = "1.0", path = "./lib_vminfo" }
//! ```
//!
//! ## Usage
//!
//! ```ignore
//!
//! // using a local client (local file cache)
//! let client: LocalClient = LocalClient::new(
//!		APP_NAME,
//!		tenant_id,
//!		client_id,
//!		Some(client_secret),
//!		vec!["sub_id1", ... "sub_idN"],
//!	)?.login_client_credentials()?;
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
//! ## License
//!
//! MIT License
//!
//! Copyright (c) His Majesty the King in Right of Canada, as represented by the minister responsible for Statistics Canada, 2022.
//!
//! Permission is hereby granted, free of charge, to any person obtaining a copy
//! of this software and associated documentation files (the "Software"), to deal
//! in the Software without restriction, including without limitation the rights
//! to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//! copies of the Software, and to permit persons to whom the Software is
//! furnished to do so, subject to the following conditions:
//!
//! The above copyright notice and this permission notice shall be included in all
//! copies or substantial portions of the Software.
//!
//! THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//! IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//! FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//! AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//! LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//! OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
//! SOFTWARE.
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

///
/// defines caching elements and modules for caching VMInfo responses
///
#[cfg(feature = "caching")]
pub mod caching {
	pub mod redis;
}

use std::fmt::Display;

use crate::query::QueryResponseType;
use crate::query::{QueryRequest, QueryResponse};
use auth::{AzCredentials, Method};
use error::{AuthErrorKind, Error, Kind, VMInfoResult};
use persistance::{FileTokenStore, PersistantStorage};
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone)]
///
/// Defines the vminfo Client
///
pub struct Client<PS>
where
	PS: PersistantStorage<AzCredentials>,
{
	tenant_id: String,
	client_id: String,
	client_secret: Option<String>,
	active_tokens: Option<AuthTokens>,
	token_store: PS,
	subscriptions: Option<Vec<String>>,
}

impl Client<FileTokenStore> {
	///
	/// creates a new Client using the 'FileTokenStore' persistence method
	///
	pub fn new(
		app_name: &str,
		tenant_id: &str,
		client_id: &str,
		client_secret: Option<String>,
		subscriptions: Option<Vec<String>>,
	) -> VMInfoResult<Self> {
		Ok(Self {
			tenant_id: String::from(tenant_id),
			client_id: String::from(client_id),
			client_secret,
			active_tokens: None,
			token_store: FileTokenStore::new(app_name)?,
			subscriptions,
		})
	}

	///
	/// creates a new vminfo Client from a persistant storage method
	///
	pub fn from_store(app_name: &str) -> VMInfoResult<Self> {
		let mut c = Self {
			tenant_id: "".to_string(),
			client_id: "".to_string(),
			client_secret: None,
			token_store: FileTokenStore::new(app_name)?,
			active_tokens: None,
			subscriptions: None,
		};

		c.load_credentials()
	}
}

impl<PS> Client<PS>
where
	PS: PersistantStorage<AzCredentials>,
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

					self.active_tokens = Some(tokens);

					self.save_credentials()?;

					Ok(self)
				}
			}
			_ => {
				let tokens = auth::login_non_interactive(&auth::Configuration::new(
					&self.tenant_id.as_str(),
					&self.client_id.as_str(),
					&self.client_secret,
				))?;

				self.active_tokens = Some(tokens);

				self.save_credentials()?;

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

					self.active_tokens = Some(tokens);

					self.save_credentials()?;

					Ok(self)
				}
			}
			_ => {
				let tokens = auth::login_interactive(&auth::Configuration::new(
					&self.tenant_id.as_str(),
					&self.client_id.as_str(),
					&None,
				))?;

				self.active_tokens = Some(tokens);

				self.save_credentials()?;

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
	pub fn save_credentials(&self) -> VMInfoResult<()> {
		if let Some(access_token) = self.access_token() {
			let client_credentials = AzCredentials {
				tenant_id: self.tenant_id.clone(),
				client_id: self.client_id.clone(),
				client_secret: self.client_secret.clone(),
				tokens: AuthTokens {
					access_token,
					refresh_token: self.refresh_token(),
				},
			};

			self.token_store.write(&client_credentials)?;

			Ok(())
		} else {
			Err(error::auth(
				None::<Error>,
				AuthErrorKind::MissingToken,
				"no access token is available from this Client",
			))
		}
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
		skip: Option<u64>,
		top: Option<u16>,
	) -> VMInfoResult<QueryResponse> {
		let resp: VMInfoResult<QueryResponse> =
			self.request(query_operand, match_regexp, show_extensions, skip, top);

		match resp {
			Ok(r) => Ok(r),
			Err(err) => match err.kind() {
				Kind::AuthenticationError(aek) => match aek {
					AuthErrorKind::MissingToken => {
						self
							.reauth()?
							.request(query_operand, match_regexp, show_extensions, skip, top)
					}
					AuthErrorKind::TokenExpired => match self.auth_method() {
						Method::ClientCredentials => {
							self
								.reauth()?
								.request(query_operand, match_regexp, show_extensions, skip, top)
						}
						Method::DeviceCode => self.clone().exchange_refresh_token()?.request(
							query_operand,
							match_regexp,
							show_extensions,
							skip,
							top,
						),
					},
					_ => Err(err)?,
				},
				_ => Err(err)?,
			},
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
		skip: Option<u64>,
		top: Option<u16>,
	) -> VMInfoResult<QueryResponse> {
		let http_client: reqwest::blocking::Client = reqwest::blocking::Client::new();

		let req_body = QueryRequest::make(
			query_operand,
			match_regexp,
			show_extensions,
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
	pub fn load_credentials(&mut self) -> VMInfoResult<Self> {
		let client_credentials = self.token_store.read()?;

		self.tenant_id = client_credentials.tenant_id;
		self.client_id = client_credentials.client_id;
		self.client_secret = client_credentials.client_secret;

		self.active_tokens = Some(AuthTokens {
			access_token: client_credentials.tokens.access_token,
			refresh_token: client_credentials.tokens.refresh_token,
		});

		Ok(self.clone())
	}
	///
	/// get an immutable access token from Client's memory
	///
	pub fn access_token(&self) -> Option<String> {
		match &self.active_tokens {
			Some(tokens) => Some(tokens.access_token.clone()),
			_ => None,
		}
	}
	///
	/// get an immutable refresh token from Client's memory
	///
	pub fn refresh_token(&self) -> Option<String> {
		match &self.active_tokens {
			Some(tokens) => tokens.refresh_token.to_owned(),
			_ => None,
		}
	}

	///
	/// will exchange a refresh token using the auth module for a new set of access and refresh tokens
	///
	pub fn exchange_refresh_token(&mut self) -> VMInfoResult<Self> {
		let rt = self.refresh_token();
		let tokens: AuthTokens = auth::exchange_refresh_tokens(&self.tenant_id, &self.client_id, rt)?;

		self.active_tokens = Some(AuthTokens {
			access_token: tokens.access_token,
			refresh_token: tokens.refresh_token,
		});

		self.save_credentials()?;

		Ok(self.clone())
	}
	///
	/// clears credentials from token/credential cache
	///
	pub fn clear_credential_cache(&self) -> VMInfoResult<()> {
		self.token_store.clear()
	}
}

impl<PS> AsMut<Client<PS>> for Client<PS>
where
	PS: PersistantStorage<AzCredentials>,
{
	fn as_mut(&mut self) -> &mut Client<PS> {
		self
	}
}

impl<PS> Display for Client<PS>
where
	PS: PersistantStorage<AzCredentials>,
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
/// defines a Client which uses local disk storage to persist token data for vminfo
///
pub type LocalClient = Client<FileTokenStore>;
