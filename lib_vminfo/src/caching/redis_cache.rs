use std::fmt::Debug;

use redis::Commands;

use crate::{
	error::{self, VMInfoResult},
	vm::VirtualMachine,
};

use super::Cache;

///
/// A results caching type that implements fields and methods for handling caching with Redis
///
#[derive(Debug, Clone)]
pub struct VMResultsCacheRedis {
	///
	/// the redis connection to use for caching storage operations
	///
	client: redis::Client,
}

impl VMResultsCacheRedis {
	///
	/// constructs a new Results Cache using Redis as the cache store
	///
	pub fn new(
		host: &str,
		port: u16,
		redis_password: Option<String>,
		use_tls: bool,
	) -> VMInfoResult<Self> {
		let uri_scheme = if use_tls { "rediss" } else { "redis" };
		let password = match redis_password {
			Some(p) => p,
			_ => String::from(""),
		};

		let redis_connection_url = format!("{}://:{}@{}:{}", uri_scheme, password, host, port);

		Ok(Self {
			client: redis::Client::open(redis_connection_url)
				.map_err(|err| error::caching(Some(err), "invalid redis connection URL"))?,
		})
	}
}

impl AsMut<VMResultsCacheRedis> for VMResultsCacheRedis {
	fn as_mut(&mut self) -> &mut VMResultsCacheRedis {
		self
	}
}

impl Cache<VirtualMachine> for VMResultsCacheRedis {
	fn put(&self, key: &str, data: &VirtualMachine) -> VMInfoResult<()> {
		let mut conn = self
			.client
			.get_connection()
			.map_err(|err| error::caching(Some(err), "failed to make connection to redis cache"))?;

		conn
			.set(key, &data)
			.map_err(|err| error::caching(Some(err), "failed to write VM results to redis cache"))?;

		Ok(())
	}

	fn get(&self, key: &str) -> VMInfoResult<VirtualMachine> {
		let mut conn = self
			.client
			.get_connection()
			.map_err(|err| error::caching(Some(err), "failed to make connection to redis cache"))?;

		Ok(conn.get(key).map_err(|err| {
			error::caching(
				Some(err),
				format!("could not find Virtual Machine with key {} in Redis", key).as_str(),
			)
		})?)
	}
}
