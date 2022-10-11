use super::error;
use chrono::{DateTime, NaiveDateTime, Utc};
use std::collections::HashMap;

/// grant type to use based on OAuth2.0 specification (https://oauth.net/2/grant-types/)
static ACCESS_GRANT_TYPE: &str = "client_credentials";
/// the resource scope for which vminfo will be used
static ACCESS_RESOURCE_SCOPE: &str = "https://management.core.windows.net/";

/// Specification for Data Transfer Object (DTO) which is used in OAuth2.0 client_crednetials flow (no user present required)
#[derive(Debug, Clone)]
pub struct AccessToken {
	pub value: String,
	pub expire_time: DateTime<Utc>,
}

impl AccessToken {
	/// performs a login using a service principle (Enterprise App or Managed Identity) and requests an access token for further authorization within the requested scopes
	///
	/// # Example
	///
	/// ```ignore
	/// let token: Result<AccessToken> = AccessToken::sp_login("tenant_id", "client_id", "client_secret");
	/// ```
	/// Where credentials specified are in the form of App ID/Client ID and password/secret to request an oauth2.0 token from the Azure API
	pub fn sp_login(
		tenant_id: &str,
		client_id: &str,
		client_secret: &str,
	) -> error::Result<AccessToken> {
		let auth_params: [(&str, &str); 4] = [
			("client_id", client_id),
			("client_secret", client_secret),
			("grant_type", ACCESS_GRANT_TYPE),
			("resource", ACCESS_RESOURCE_SCOPE),
		];

		let rest_client: reqwest::blocking::Client = reqwest::blocking::Client::new();

		let resp: HashMap<String, String> = rest_client
			.get(format!(
				"https://login.microsoftonline.com/{}/oauth2/token",
				tenant_id
			))
			.form(&auth_params)
			.send()
			.map_err(|err| {
        match err.status().unwrap_or(reqwest::StatusCode::INTERNAL_SERVER_ERROR) {
          reqwest::StatusCode::NOT_FOUND => error::sp_login(err, "requested authorization endpoint does not exist. this could be a problem with the vminfo library"),
          reqwest::StatusCode::UNAUTHORIZED => error::sp_login(err, "invalid client ID and password were supplied"),
          reqwest::StatusCode::FORBIDDEN => error::sp_login(err, format!("the user with client ID, {}, does not posess the correct permissions to authenticate", client_id).as_str()),
          _ => error::sp_login(err, "some unknown authentication request error ocurred"), 
        }
      })?
			.json::<HashMap<String, String>>()
			.map_err(|err| error::sp_login(err, format!(
				"could not parse access token response from HTTP response.",
			)
			.as_str()))?;

		let timestamp = resp
			.get("expires_on")
			.unwrap()
			.parse::<i64>()
			.map_err(|err| {
				error::sp_login(
					err,
					format!("could not parse access_token expiry from HTTP response data.",).as_str(),
				)
			})?;

		let native_time = NaiveDateTime::from_timestamp(timestamp, 0);
		let datetime: DateTime<Utc> = DateTime::from_utc(native_time, Utc);

		Ok(AccessToken {
			value: resp.get("access_token").unwrap().to_owned(),
			expire_time: datetime,
		})
	}
}

impl std::fmt::Display for AccessToken {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.value)
	}
}
