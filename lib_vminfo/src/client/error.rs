#[allow(unused)]
#[allow(unused_must_use)]
use std::error::Error as StdError;
use std::result::Result as StdResult;

#[derive(Debug)]
pub enum Kind {
	AccessTokenExpired,
	AccessTokenSPLogin,
	VMNotFound,
	VMRequest,
}

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug)]
pub struct Error {
	inner: Box<Inner>,
}

pub type BoxError = Box<dyn StdError + Send + Sync>;

#[derive(Debug)]
pub struct Inner {
	kind: Kind,
	source: Option<BoxError>,
	message: String,
}

impl Error {
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
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let _: std::result::Result<(), std::fmt::Error> = match self.inner.kind {
			Kind::AccessTokenSPLogin => f.write_str("sp login error"),
			Kind::AccessTokenExpired => f.write_str("access token expired"),
			Kind::VMRequest => f.write_str("request error"),
			Kind::VMNotFound => f.write_str("vm(s) not found"),
		};

		write!(f, ": {}", self.inner.message)
	}
}

impl StdError for Error {
	fn source(&self) -> Option<&(dyn StdError + 'static)> {
		self.inner.source.as_ref().map(|e| &**e as _)
	}
}

pub fn sp_login<E: Into<BoxError>>(e: E, message: &str) -> Error {
	Error::new(Kind::AccessTokenSPLogin, Some(e), message)
}

pub fn vm_request<E: Into<BoxError>>(e: E, message: &str) -> Error {
	Error::new(Kind::VMRequest, Some(e), message)
}
