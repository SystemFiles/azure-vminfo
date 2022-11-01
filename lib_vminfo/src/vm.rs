use serde::de::Deserializer;
use serde::{Deserialize, Serialize};

/// Defines the fields that a host result should contain.
/// This is Serializable from the Resource Graph response and to json for consumption outside of vminfo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualMachine {
	#[serde(alias = "vmId", rename(serialize = "vmId"))]
	vm_id: Option<String>,
	#[serde(alias = "vmName", rename(serialize = "vmName"))]
	vm_name: Option<String>,
	created: Option<String>,
	sub: Option<String>,
	location: Option<String>,
	rg: Option<String>,
	#[serde(
		alias = "privateIp",
		rename(serialize = "privateIp"),
		deserialize_with = "parse_ipv4_address"
	)]
	private_ip: std::net::Ipv4Addr,
	#[serde(alias = "osType", rename(serialize = "osType"))]
	os_type: Option<String>,
	#[serde(alias = "osName", rename(serialize = "osName"))]
	os_name: Option<String>,
	#[serde(alias = "osVersion", rename(serialize = "osVersion"))]
	os_version: Option<String>,
	powerstate: Option<String>,
	#[serde(alias = "vmSize", rename(serialize = "vmSize"))]
	vm_size: Option<String>,
	#[serde(alias = "virtualNetwork", rename(serialize = "virtualNetwork"))]
	virtual_network: Option<String>,
	subnet: Option<String>,
	#[serde(default)]
	extensions: Vec<VirtualMachineExtension>,
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
		}
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

// TODO: implement custom extensions deserializer that is more accepting of null keys in extension lists

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
