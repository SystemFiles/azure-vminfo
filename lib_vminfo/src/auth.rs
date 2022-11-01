use super::error::{auth, client_config, Error, VMInfoResult};
use crate::error::AuthErrorKind;
use crate::AuthTokens;
use oauth2::basic::{
	BasicErrorResponse, BasicRevocationErrorResponse, BasicTokenIntrospectionResponse, BasicTokenType,
};
use oauth2::devicecode::StandardDeviceAuthorizationResponse;
use oauth2::{
	basic::BasicClient, reqwest::http_client, AccessToken, AuthUrl, ClientId, ClientSecret,
	ExtraTokenFields, RefreshToken, Scope, TokenResponse, TokenType, TokenUrl,
};
use oauth2::{
	helpers, Client, DeviceAuthorizationUrl, EmptyExtraTokenFields, StandardRevocableToken,
	StandardTokenResponse,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

///
/// Custom Token Response type to replace the StandardTokenResponse provided by oauth2-rs. This is required because Microsoft is not in compliance with the RFC spec for oauth2.0
///
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AzureTokenResponse<EF, TT>
where
	EF: ExtraTokenFields,
	TT: TokenType,
{
	access_token: AccessToken,
	#[serde(bound = "TT: TokenType")]
	#[serde(deserialize_with = "helpers::deserialize_untagged_enum_case_insensitive")]
	token_type: TT,
	#[serde(skip_serializing_if = "Option::is_none")]
	expires_in: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	refresh_token: Option<RefreshToken>,
	#[serde(rename = "scope")]
	#[serde(deserialize_with = "helpers::deserialize_space_delimited_vec")]
	#[serde(serialize_with = "helpers::serialize_space_delimited_vec")]
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(default)]
	scopes: Option<Vec<Scope>>,

	#[serde(bound = "EF: ExtraTokenFields")]
	#[serde(flatten)]
	extra_fields: EF,
}

impl<EF, TT> AzureTokenResponse<EF, TT>
where
	EF: ExtraTokenFields,
	TT: TokenType,
{
	///
	/// Creates a new AzureTokenResponse
	///
	pub fn new(access_token: AccessToken, token_type: TT, extra_fields: EF) -> Self {
		Self {
			access_token,
			token_type,
			expires_in: None,
			refresh_token: None,
			scopes: None,
			extra_fields,
		}
	}

	///
	/// Sets the value of the access token in an instance of the 'AzureTokenResponse'
	///
	pub fn set_access_token(&mut self, access_token: AccessToken) {
		self.access_token = access_token;
	}

	///
	/// Sets the token type for an instance of the 'AzureTokenResponse'
	///
	pub fn set_token_type(&mut self, token_type: TT) {
		self.token_type = token_type;
	}

	///
	/// set the expire time for an instance of the 'AzureTokenResponse'
	///
	pub fn set_expires_in(&mut self, expires_in: Option<&Duration>) {
		self.expires_in = expires_in.map(|exp| Duration::as_secs(exp).to_string());
	}

	///
	/// set the value for the refresh token for an instance of the 'AzureTokenResponse'
	///
	pub fn set_refresh_token(&mut self, refresh_token: Option<RefreshToken>) {
		self.refresh_token = refresh_token;
	}

	///
	/// set the scopes for an 'AzureTokenResponse'
	///
	pub fn set_scopes(&mut self, scopes: Option<Vec<Scope>>) {
		self.scopes = scopes;
	}

	///
	/// get any extra fields for an 'AzureTokenResponse'
	///
	pub fn extra_fields(&self) -> &EF {
		&self.extra_fields
	}

	///
	/// set any extra fields for an 'AzureTokenResponse'
	///
	pub fn set_extra_fields(&mut self, extra_fields: EF) {
		self.extra_fields = extra_fields;
	}
}

impl<EF, TT> TokenResponse<TT> for AzureTokenResponse<EF, TT>
where
	EF: ExtraTokenFields,
	TT: TokenType,
{
	///
	/// The access token issued by the Azure authentication server
	///
	fn access_token(&self) -> &AccessToken {
		&self.access_token
	}
	///
	/// get the token type for an 'AzureTokenResponse'
	///
	fn token_type(&self) -> &TT {
		&self.token_type
	}
	///
	/// get the expire time for an 'AzureTokenResponse' as a 'Duration'
	///
	fn expires_in(&self) -> Option<Duration> {
		self.expires_in.as_ref().map(|exp| {
			let expires_in_number: u64 = exp.parse::<u64>().unwrap();

			Duration::from_secs(expires_in_number)
		})
	}
	///
	/// get the associated refresh token for an 'AzureTokenResponse'
	///
	fn refresh_token(&self) -> Option<&RefreshToken> {
		self.refresh_token.as_ref()
	}
	///
	/// get the scopes for an 'AzureTokenResponse'
	///
	fn scopes(&self) -> Option<&Vec<Scope>> {
		self.scopes.as_ref()
	}
}

impl<EF, TT> From<StandardTokenResponse<EF, TT>> for AzureTokenResponse<EF, TT>
where
	EF: ExtraTokenFields + Clone,
	TT: TokenType,
{
	///
	/// defines a method to perform a conversion from a StandardTokenResponse to the custom 'AzureTokenResponse'
	///
	fn from(st: StandardTokenResponse<EF, TT>) -> Self {
		let expire_time_string = st
			.expires_in()
			.map(|exp| Duration::as_secs(&exp).to_string());

		let extra_fields: EF = st.extra_fields().clone();

		AzureTokenResponse {
			access_token: st.access_token().clone(),
			token_type: st.token_type().clone(),
			expires_in: expire_time_string,
			refresh_token: st.refresh_token().map(|r| r.clone()),
			scopes: st.scopes().map(|s| s.clone()),
			extra_fields: extra_fields,
		}
	}
}

///
/// alias for AzureTokenResponse type
///
pub type BasicAzureTokenResponse = AzureTokenResponse<EmptyExtraTokenFields, BasicTokenType>;

///
/// Alias for Client that makes use of the AzureTokenResponse custom type
/// Defines a custom token response type ('AzureTokenResponse') for the oauth2 client
///
/// Required since Microsoft is not in compliance with the RFC oauth2.0 spec
///
pub type AzureClient = Client<
	BasicErrorResponse,
	BasicAzureTokenResponse,
	BasicTokenType,
	BasicTokenIntrospectionResponse,
	StandardRevocableToken,
	BasicRevocationErrorResponse,
>;

///
/// Defines the list of available authentication methods supported by lib_vminfo
///
#[derive(Debug, Clone, PartialEq)]
pub enum Method {
	/// Devicecode interactive authentication method as defined by [RFC-8628](https://www.rfc-editor.org/rfc/rfc8628#section-3.4)
	DeviceCode,
	/// Client Credentials non-interactive authentication method as defined by [RFC-6749](https://www.rfc-editor.org/rfc/rfc6749#section-4.4)
	ClientCredentials,
}

impl std::fmt::Display for Method {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match *self {
			Method::DeviceCode => write!(f, "DeviceCode"),
			Method::ClientCredentials => write!(f, "ClientCredentials"),
		}
	}
}

///
/// Defines a Azure Credential object which contains necessary credentials for authentication with the Azure Resource Graph.
/// This object must be in compliance with credential storage requirements for the purposes of persisting login data across sessions
///
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct AzCredentials {
	/// Azure tenant ID
	pub tenant_id: String,
	/// Azure Client ID
	pub client_id: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	/// OPTIONAL: Azure Client Password
	pub client_secret: Option<String>,
	/// Relevant Azure Access and Refresh Tokens
	pub tokens: AuthTokens,
}

///
/// Authentication configuration object
///
#[derive(Debug, Clone)]
pub struct Configuration {
	/// Azure tenant ID
	pub tenant_id: String,
	/// Azure app/user client ID
	pub client_id: String,
	/// (optionally) an Azure client_secret / password used for non-interactive authentication
	pub client_secret: Option<String>,
	/// A list of resource/API scopes to ask for from the authorization server
	pub scopes: Vec<Scope>,
}

impl Configuration {
	///
	/// creates a new Authentication configuration object
	///
	pub fn new(tenant_id: &str, client_id: &str, client_secret: &Option<String>) -> Self {
		Self {
			tenant_id: tenant_id.to_string(),
			client_id: client_id.to_string(),
			client_secret: client_secret.to_owned(),
			scopes: Configuration::default().scopes,
		}
	}
}

impl Default for Configuration {
	fn default() -> Self {
		Self {
			tenant_id: "XXX".to_string(),
			client_id: "XXX".to_string(),
			client_secret: Some("XXX".to_string()),
			scopes: vec![Scope::new(
				"https://management.core.windows.net/".to_string(),
			)],
		}
	}
}

///
/// performs a non-interactive login using a client_id and password (secret)
///
pub fn login_non_interactive(conf: &Configuration) -> VMInfoResult<AuthTokens> {
	let token_url: String = format!(
		"https://login.microsoftonline.com/{}/oauth2/token",
		conf.tenant_id
	);

	let client_secret: Option<ClientSecret> = match &conf.client_secret {
		Some(secret) => Some(ClientSecret::new(secret.clone())),
		_ => None,
	};

	let client = AzureClient::new(
		ClientId::new(conf.client_id.clone()),
		client_secret,
		AuthUrl::new("http://authorize/".to_string()).map_err(|err| {
			auth(
				Some(err),
				AuthErrorKind::BadRequest,
				"could not parse authorization url. it is likely invalid",
			)
		})?,
		Some(TokenUrl::new(token_url).map_err(|err| {
			auth(
				Some(err),
				AuthErrorKind::BadRequest,
				"could not parse token url. it is likely invalid",
			)
		})?),
	);

	let token_result = client
		.exchange_client_credentials()
		.add_extra_param("resource", "https://management.core.windows.net/")
		.request(http_client)
		.map_err(|err| {
			auth(
				Some(err),
				AuthErrorKind::BadCredentials,
				"invalid tenant_id and client_id or secret combination provided",
			)
		})?;

	Ok(AuthTokens {
		access_token: token_result.access_token().secret().to_owned(),
		refresh_token: match token_result.refresh_token() {
			Some(rt) => Some(rt.secret().to_owned()),
			_ => None,
		},
	})
}

///
/// performs an interactive login provided a client_id and login challenge
///
pub fn login_interactive(conf: &Configuration) -> VMInfoResult<AuthTokens> {
	let token_url: String = format!(
		"https://login.microsoftonline.com/{}/oauth2/v2.0/token",
		conf.tenant_id
	);

	let device_code_url = DeviceAuthorizationUrl::new(format!(
		"https://login.microsoftonline.com/{}/oauth2/v2.0/devicecode",
		conf.tenant_id
	))
	.map_err(|err| {
		client_config(
			Some(err),
			"failed to create a valid device authorization URL for Azure Oauth2.0",
		)
	})?;

	let client = BasicClient::new(
		ClientId::new(conf.client_id.clone()),
		None,
		AuthUrl::new("http://authorize/".to_string())
			.map_err(|err| client_config(Some(err), "authorization URL config is not valid"))?,
		Some(TokenUrl::new(token_url).map_err(|err| {
			auth(
				Some(err),
				AuthErrorKind::BadRequest,
				"could not parse token url. it is likely invalid",
			)
		})?),
	)
	.set_device_authorization_url(device_code_url);

	let details: StandardDeviceAuthorizationResponse = client
		.exchange_device_code()
		.map_err(|err| {
			auth(
				Some(err),
				AuthErrorKind::BadRequest,
				"failed to configure exchange device code for interactive authentication",
			)
		})?
		.add_scopes(vec![
			Scope::new("https://management.core.windows.net/user_impersonation".to_string()),
			Scope::new("offline_access".to_string()),
		])
		.request(http_client)
		.map_err(|err| {
			auth(
				Some(err),
				AuthErrorKind::BadRequest,
				"failed to get device code details for interactive authentication",
			)
		})?;

	println!(
		"Open this URL in your browser:\n{}\nand enter the code: {}",
		details.verification_uri().to_string(),
		details.user_code().secret().to_string()
	);

	let token_req_result = client
		.exchange_device_access_token(&details)
		.add_extra_param("code", details.device_code().secret().to_string())
		.request(http_client, std::thread::sleep, None);

	let token_result = token_req_result.map_err(|err| {
		auth(
			Some(err),
			AuthErrorKind::BadRequest,
			"could not authenticate user with devicecode auth",
		)
	})?;

	Ok(AuthTokens {
		access_token: token_result.access_token().secret().to_owned(),
		refresh_token: match token_result.refresh_token() {
			Some(rt) => Some(rt.secret().to_owned()),
			_ => None,
		},
	})
}

///
/// performs a token refresh provided a valid refresh token
///
pub fn exchange_refresh_tokens(
	tenant_id: &str,
	client_id: &str,
	refresh_token: Option<String>,
) -> VMInfoResult<AuthTokens> {
	let token_url: String = format!(
		"https://login.microsoftonline.com/{}/oauth2/token",
		tenant_id
	);

	let client = AzureClient::new(
		ClientId::new(client_id.to_string()),
		None,
		AuthUrl::new("http://authorize/".to_string())
			.map_err(|err| client_config(Some(err), "authorization URL config is not valid"))?,
		Some(TokenUrl::new(token_url).map_err(|err| {
			auth(
				Some(err),
				AuthErrorKind::BadRequest,
				"could not parse token url. it is likely invalid",
			)
		})?),
	);

	let mut token_result: AzureTokenResponse<EmptyExtraTokenFields, BasicTokenType> =
		AzureTokenResponse::new(
			AccessToken::new("s".to_string()),
			BasicTokenType::Bearer,
			EmptyExtraTokenFields {},
		);
	if let Some(rt) = refresh_token {
		token_result = client
			.exchange_refresh_token(&RefreshToken::new(rt))
			.request(http_client)
			.map_err(|err| {
				auth(
					Some(err),
					AuthErrorKind::BadRefresh,
					"refresh token provided could not be used to obtain a new access token",
				)
			})?;
	} else {
		Err(auth(
			None::<Error>,
			AuthErrorKind::BadRefresh,
			"access token is expired and failed to exchange refresh token. reauthentication is required",
		))?
	}

	Ok(AuthTokens {
		access_token: token_result.access_token().secret().to_owned(),
		refresh_token: match token_result.refresh_token() {
			Some(rt) => Some(rt.secret().to_owned()),
			_ => Err(auth(
				None::<Error>,
				AuthErrorKind::MissingToken,
				"no refresh token supplied with login ... this is unusable",
			))?,
		},
	})
}
