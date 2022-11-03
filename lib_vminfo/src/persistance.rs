//!
//!
//! Provides traits / structs / types related to persisting authentication details (tokens / client credentials)
//!
//! Other credential formats will work as well ... just implement the PersistantStorage trait using your own type
//!
//!

use std::{
	fmt::Display,
	fs::{self, File},
	io::Write,
	path::PathBuf,
};

use serde::{de::DeserializeOwned, Serialize};
use std::str::FromStr;

use crate::{
	auth::AzCredentials,
	error::{self, AuthErrorKind, VMInfoResult},
};

///
/// defines common methods for a persistant storage solution for storing Access and Refresh Tokens.
///
pub trait PersistantStorage<DT>: Clone + Display
where
	DT: Serialize + DeserializeOwned + Clone,
{
	///
	/// defines a method for writing / storing a pair of access and refresh tokens
	///
	fn write(&self, data: &DT) -> VMInfoResult<()>;
	///
	/// defines a method for reading access and refresh tokens from a persistant storage solution
	///
	fn read(&self) -> VMInfoResult<DT>;
	///
	/// defines a method for clearing out the local credential and token cache
	///
	/// **note**: this WILL prevent the requests from being processed and will require authentication
	///
	fn clear(&self) -> VMInfoResult<()>;
}

///
/// A Persistence Method for storage of Access and Refresh token pairs
///
#[derive(Debug, Clone)]
pub struct FileTokenStore {
	file_path: PathBuf,
}

impl FileTokenStore {
	///
	/// creates a new FileTokenStore
	///
	#[cfg(target_os = "macos")]
	pub fn new(app_name: &str) -> VMInfoResult<FileTokenStore> {
		let username: String = String::from(users::get_current_username().unwrap().to_str().unwrap());

		#[cfg(target_os = "macos")]
		let path = PathBuf::from_str(
			format!(
				"/Users/{}/Library/Application Support/{}/tokens.json",
				username, app_name
			)
			.as_str(),
		)
		.map_err(|err| {
			error::client_config(Some(err), "failed to generate path for token persistence")
		})?;

		let store = Self { file_path: path };
		store.create_config()?;

		if !store.file_path.exists() {
			let _: File = File::create(&store.file_path)
				.map_err(|err| error::client_config(Some(err), "failed to create token storage file"))?;
		}

		Ok(store)
	}

	///
	/// creates a new FileTokenStore
	///
	#[cfg(target_os = "linux")]
	pub fn new(app_name: &str) -> VMInfoResult<FileTokenStore> {
		let username: String = String::from(users::get_current_username().unwrap().to_str().unwrap());

		let path = match username.as_str() {
			"root" => {
				PathBuf::from_str(format!("/{}/.config/{}/tokens.json", username, app_name).as_str())
					.map_err(|err| {
						error::client_config(Some(err), "failed to generate path for token persistence")
					})?
			}
			_ => {
				PathBuf::from_str(format!("/home/{}/.config/{}/tokens.json", username, app_name).as_str())
					.map_err(|err| {
					error::client_config(Some(err), "failed to generate path for token persistence")
				})?
			}
		};

		let store = Self { file_path: path };
		store.create_config()?;

		Ok(store)
	}

	fn create_config(&self) -> VMInfoResult<()> {
		fs::create_dir_all(&self.file_path.parent().unwrap())
			.map_err(|err| error::client_config(Some(err), "failed to create config directory path"))?;

		Ok(())
	}
}

impl PersistantStorage<AzCredentials> for FileTokenStore {
	fn write(&self, data: &AzCredentials) -> VMInfoResult<()> {
		if !self.file_path.parent().unwrap().exists() {
			self.create_config()?
		}

		let mut tokens_file: File = File::create(&self.file_path)
			.map_err(|err| error::other(Some(err), "failed to create token storage file"))?;
		tokens_file
			.write(
				serde_json::to_string_pretty(&data)
					.map_err(|err| {
						error::other(
							Some(err),
							"failed to generate JSON for auth tokens persistence",
						)
					})?
					.as_bytes(),
			)
			.map_err(|err| error::other(Some(err), "failed to write auth tokens to file"))?;

		Ok(())
	}

	fn read(&self) -> VMInfoResult<AzCredentials> {
		let contents = fs::read_to_string(&self.file_path).map_err(|err| {
			error::auth(
				Some(err),
				AuthErrorKind::MissingToken,
				"could not read credentials from file.",
			)
		})?;

		Ok(
			serde_json::from_str::<AzCredentials>(&contents.as_str()).map_err(|err| {
				error::auth(
					Some(err),
					AuthErrorKind::BadCredentials,
					"could not parse credential contents to struct",
				)
			})?,
		)
	}

	fn clear(&self) -> VMInfoResult<()> {
		if !self.file_path.parent().unwrap().exists() {
			Ok(())
		} else {
			let _ = File::create(&self.file_path).map_err(|err| {
				error::other(
					Some(err),
					"could not truncate local token/credential cache file",
				)
			})?;
			Ok(())
		}
	}
}

impl Display for FileTokenStore {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"Token Secret File Located at: {}",
			self.file_path.as_path().to_str().unwrap_or("unknown")
		)
	}
}
