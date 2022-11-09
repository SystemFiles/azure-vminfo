///
/// provides a concrete redis cache
///
pub mod redis_cache;
use serde::{de::DeserializeOwned, Serialize};

use crate::error::VMInfoResult;

///
/// types that implement the Cache trait will store key-value data using some cache mechanism
/// and also provide an easy API for retrieving values
///
pub trait Cache<DT>
where
	DT: Serialize + DeserializeOwned + Clone,
{
	///
	/// store a value or update an existing cached value
	///
	fn put(&self, key: &str, data: &DT) -> VMInfoResult<()>;
	///
	/// retrieve a stored cached value (if one exists) - Error if None exists
	///
	fn get(&self, key: &str) -> VMInfoResult<DT>;
}
