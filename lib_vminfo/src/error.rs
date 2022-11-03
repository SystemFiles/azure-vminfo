//!
//!
//! Provides common error types for the vminfo library
//!
//!
#[allow(unused)]
#[allow(unused_must_use)]
use super::auth::Method;

use std::error::Error as StdError;
use std::result::Result as StdResult;

///
/// Describes the various kinds of authentication / authorization errors that may arise during standard use of the vminfo library
///
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum AuthErrorKind {
	///
	/// User submitted credentials that were not accepted
	///
	BadCredentials,
	///
	/// Either an Access token or Refresh token is missing
	///
	MissingToken,
	///
	/// The provided Access token is expired
	///
	TokenExpired,
	///
	/// Could not refresh token is expired or was unable to obtain a new access token for some other reason
	///
	BadRefresh,
	///
	/// Bad authentication configuration format / content
	///
	BadRequest,
	///
	/// Permissions not valid
	///
	AccessDenied,
}

impl From<AuthErrorKind> for reqwest::StatusCode {
	fn from(k: AuthErrorKind) -> Self {
		match k {
			AuthErrorKind::AccessDenied => reqwest::StatusCode::FORBIDDEN,
			AuthErrorKind::BadCredentials => reqwest::StatusCode::UNAUTHORIZED,
			AuthErrorKind::BadRefresh => reqwest::StatusCode::UNAUTHORIZED,
			AuthErrorKind::MissingToken => reqwest::StatusCode::UNAUTHORIZED,
			AuthErrorKind::TokenExpired => reqwest::StatusCode::UNAUTHORIZED,
			AuthErrorKind::BadRequest => reqwest::StatusCode::BAD_REQUEST,
		}
	}
}

impl std::fmt::Display for AuthErrorKind {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match *self {
			Self::BadCredentials => write!(f, "Bad credentials provided"),
			Self::MissingToken => write!(f, "Access or refresh token missing"),
			Self::TokenExpired => write!(f, "Access token is expired"),
			Self::BadRefresh => write!(f, "Failed to refresh access"),
			Self::BadRequest => write!(f, "Bad authentication / authorization request"),
			Self::AccessDenied => write!(f, "Access denied"),
		}
	}
}

///
/// Describes the various kinds of errors that may appear when using lib-vminfo
///
#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
	///
	/// Error thrown when client configuration fails or is invalid
	///
	ClientCreateError,
	///
	/// Error thrown if there is some issue with client authentication
	///
	AuthenticationError(AuthErrorKind),
	///
	/// Error thrown if the VM result from the client is empty and contains no results
	///
	NoneFoundError,
	///
	/// Error thrown if there was a problem initiating a request for VM info from Resource Graph. Takes a reqwest status_code for extra context
	///
	RequestError(Option<reqwest::StatusCode>),
	///
	/// Error thrown if there is not sufficient information to determine what the error was that occurred
	///
	Other,
}

///
/// Alias for any error that might occur when using the vminfo crate.
///
pub type VMInfoResult<T> = StdResult<T, Error>;

///
/// the custom error type for anything that might go wrong using the vminfo client
///
#[derive(Debug)]
pub struct Error {
	inner: Box<Inner>,
}

///
/// an alias for the source of a thrown custom error
///
pub type BoxError = Box<dyn StdError + Send + Sync>;

///
/// defines properties of the wrapped error
///
#[derive(Debug)]
pub struct Inner {
	kind: Kind,
	source: Option<BoxError>,
	message: String,
}

impl Error {
	///
	/// builds a new instance of the custom vminfo Error type
	///
	pub fn new<E>(kind: Kind, source: Option<E>, message: &str) -> Error
	where
		E: Into<BoxError>,
	{
		Error {
			inner: Box::new(Inner {
				kind,
				source: source.map(Into::into),
				message: message.to_string(),
			}),
		}
	}

	///
	/// used to get the kind of error
	///
	pub fn kind(&self) -> Kind {
		self.inner.kind.clone()
	}

	///
	/// used to get a reference to the wrapped error of the custom vminfo Error type
	///
	pub fn inner(&self) -> Option<&dyn StdError> {
		self.source()
	}
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let _: std::result::Result<(), std::fmt::Error> = match self.inner.kind {
			Kind::ClientCreateError => f.write_str("client creation error"),
			Kind::AuthenticationError(aek) => {
				f.write_str(format!("authentication error ({})", aek).as_str())
			}
			Kind::NoneFoundError => f.write_str("no vm found error"),
			Kind::RequestError(s) => f.write_str(
				format!(
					"HTTP request error. {}",
					if s.is_none() {
						"NO_STATUS".to_string()
					} else {
						s.unwrap().to_string()
					}
				)
				.as_str(),
			),
			Kind::Other => f.write_str("unknown error"),
		};

		write!(f, ": {}", self.inner.message)
	}
}

impl StdError for Error {
	fn source(&self) -> Option<&(dyn StdError + 'static)> {
		self.inner.source.as_ref().map(|e| &**e as _)
	}
}

///
/// builds an error for instances where authentication fails and no access token can be obtained
///
pub fn auth<E: Into<BoxError>>(e: Option<E>, kind: AuthErrorKind, message: &str) -> Error {
	Error::new(Kind::AuthenticationError(kind), e, message)
}

///
/// builds an error for capturing instances when the provided client configuration for the vminfo client is not valid
///
pub fn client_config<E: Into<BoxError>>(e: Option<E>, message: &str) -> Error {
	Error::new(Kind::ClientCreateError, e, message)
}

///
/// builds a custom none found error for instances where no VMs are returned from a successful vminfo request
///
pub fn none_found<E: Into<BoxError>>(e: Option<E>, message: &str) -> Error {
	Error::new(Kind::NoneFoundError, e, message)
}
///
/// builds a request error taking an optional status code from the reqwest client
///
pub fn request<E: Into<BoxError>>(
	e: Option<E>,
	req_status: Option<reqwest::StatusCode>,
	message: &str,
) -> Error {
	Error::new(Kind::RequestError(req_status), e, message)
}

///
/// builds an error type for unknown errors that might appear during vminfo API processes
///
pub fn other<E: Into<BoxError>>(e: Option<E>, message: &str) -> Error {
	Error::new(Kind::Other, e, message)
}
