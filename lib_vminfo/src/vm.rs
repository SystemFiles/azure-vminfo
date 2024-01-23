//!
//!
//! Provides a model for Virtual Machines
//!
//!
use std::io;

use redis::{from_redis_value, FromRedisValue, ToRedisArgs};
use serde::de::Deserializer;
use serde::{Deserialize, Serialize};

///
/// Defines the fields that a host result should contain.
/// This is Serializable from the Resource Graph response and to json for consumption outside of vminfo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualMachine {
	///
	/// The ID that uniquely identifies this Virtual Machine
	///
	#[serde(alias = "vmId", rename(serialize = "vmId"))]
	vm_id: Option<String>,
	///
	/// The name of the Virtual Machine
	///
	#[serde(alias = "vmName", rename(serialize = "vmName"))]
	pub vm_name: Option<String>,
	///
	/// The create timestamp that identifies when the Virtual Machine was created
	///
	created: Option<String>,
	///
	/// The subscription that this Virtual Machine resides in
	///
	sub: Option<String>,
	///
	/// The datacentre location where this Virtual Machine resides
	///
	location: Option<String>,
	///
	/// The resource group which this Virtual Machine resource resides in
	///
	rg: Option<String>,
	///
	/// The IP address for the Virtual Machine
	///
	#[serde(
		alias = "privateIp",
		rename(serialize = "privateIp"),
		deserialize_with = "parse_ipv4_address"
	)]
	private_ip: std::net::Ipv4Addr,
	///
	/// The OS Type for this Virtual Machine (can be: Linux or Windows)
	///
	#[serde(alias = "osType", rename(serialize = "osType"))]
	os_type: Option<String>,
	///
	/// The OS Distribution Name for this Virtual Machine (ie: Ubuntu, RedHat, etc.)
	///
	#[serde(alias = "osName", rename(serialize = "osName"))]
	os_name: Option<String>,
	///
	/// The version fo the OS Distribution being run on the Virtual Machine
	///
	#[serde(alias = "osVersion", rename(serialize = "osVersion"))]
	os_version: Option<String>,
	///
	/// The current power state for this Virtual Machine
	///
	powerstate: Option<String>,
	///
	/// The VM size specification as defined by Azure in their [vmsize documentation](https://learn.microsoft.com/en-us/azure/virtual-machines/sizes)
	///
	#[serde(alias = "vmSize", rename(serialize = "vmSize"))]
	vm_size: Option<String>,
	///
	/// The primary Azure VNet that this Virtual Machine is connected to
	///
	#[serde(alias = "virtualNetwork", rename(serialize = "virtualNetwork"))]
	virtual_network: Option<String>,
	///
	/// The primary Azure subnet that this Virtual Machine is connected to
	///
	subnet: Option<String>,
	///
	/// A List of Azure Virtual Machine Extensions that are installed for this VM (None if not requested)
	///
	#[serde(default)]
	extensions: Vec<VirtualMachineExtension>,
	///
	/// A list of Azure resource tags associated with an Azure Virtual Machine
	/// 
	#[serde(default)]
	tags: Vec<AzureTag>,
}

impl Default for VirtualMachine {
	fn default() -> Self {
		Self {
			vm_id: None,
			vm_name: None,
			created: None,
			sub: None,
			location: None,
			rg: None,
			private_ip: std::net::Ipv4Addr::new(0, 0, 0, 0),
			os_type: None,
			os_name: None,
			os_version: None,
			powerstate: None,
			vm_size: None,
			virtual_network: None,
			subnet: None,
			extensions: vec![],
			tags: vec![],
		}
	}
}

impl ToRedisArgs for VirtualMachine {
	fn to_redis_args(&self) -> Vec<Vec<u8>> {
		let v: Vec<u8> = serde_json::to_string(self)
			.expect("cannot convert Virtual Machine to redis args")
			.as_bytes()
			.into_iter()
			.map(|i| *i)
			.collect();

		vec![v]
	}
	fn write_redis_args<W>(&self, out: &mut W)
	where
		W: ?Sized + redis::RedisWrite,
	{
		let vm: VirtualMachine = self.clone();
		let vm_str = serde_json::to_string(&vm).expect("cannot convert Virtual Machine to redis args");

		// convert VM JSON to bytes
		let vm_bytes: &[u8] = vm_str.as_bytes();

		out.write_arg(vm_bytes)
	}
}

impl FromRedisValue for VirtualMachine {
	fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
		match v {
			redis::Value::Data(d) => Ok(
				serde_json::from_slice::<VirtualMachine>(d).map_err(|err| redis::RedisError::from(err))?,
			),
			_ => Err(redis::RedisError::from(io::Error::new(
				io::ErrorKind::InvalidData,
				"Cannot read data into VirtualMachine type",
			))),
		}
	}
	fn from_redis_values(items: &[redis::Value]) -> redis::RedisResult<Vec<Self>> {
		let mut res: Vec<VirtualMachine> = Vec::new();

		for (_, v) in items.into_iter().enumerate() {
			res.push(from_redis_value(v)?);
		}

		Ok(res)
	}
}

///
/// deserializer that will take a JSON response as a string and pull out a valid IPv4 address
/// if errors occur, will produce a default `0.0.0.0` address in the resulting struct
///
fn parse_ipv4_address<'de, D>(d: D) -> Result<std::net::Ipv4Addr, D::Error>
where
	D: Deserializer<'de>,
{
	match Deserialize::deserialize(d)
		.map(|x: Option<std::net::Ipv4Addr>| x.unwrap_or(std::net::Ipv4Addr::new(0, 0, 0, 0)))
	{
		Ok(r) => Ok(r),
		Err(_) => Ok(std::net::Ipv4Addr::new(0, 0, 0, 0)),
	}
}

// TODO: implement custom extensions deserializer that is more accepting of null keys in extension lists ([Github Issue](https://github.com/SystemFiles/azure-vminfo/issues/1))

/// Describes a virtual machine extension in Azure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualMachineExtension {
	name: String,
	version: String,
}

impl Default for VirtualMachineExtension {
	fn default() -> Self {
		Self {
			name: "XXX".to_string(),
			version: "XXX".to_string(),
		}
	}
}

/// Describes Azure resource tags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct  AzureTag {
	key: String,
	value: String
}

impl Default for AzureTag {
	fn default() -> Self {
		Self {
			key: "KEY_X".to_string(),
			value: "VAL_X".to_string(),
		}
	}
}
