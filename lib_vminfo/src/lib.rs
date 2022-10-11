pub mod client {
	pub mod api;
	mod auth;
	mod error;
}

pub mod models {
	pub mod query;
	pub mod vm;
}

#[cfg(feature = "caching")]
pub mod caching {
	pub mod redis;
}
