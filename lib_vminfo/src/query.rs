//!
//!
//! Provides request and response types for vminfo queries to the Azure Resource Graph API
//!
//!

use super::vm::VirtualMachine;
use redis::ToRedisArgs;
use serde::{Deserialize, Serialize};

/// specifies and acceptable request body format for Resource Graph to understand
/// QueryRequest is serialized into raw JSON when passed into the HTTP request body
#[derive(Debug, Serialize, Deserialize)]
pub struct QueryRequest {
	query: String,
	options: QueryRequestOptions,
	subscriptions: Option<Vec<String>>,
}

impl QueryRequest {
	/// builds a request body for an opinionated use of the Resource Graph API. this constructor will template a valid KQL query which can be passed to the Resource Graph API
	/// and will include response format and quantity parameters as specified.
	///
	/// # Example
	///
	/// ```ignore
	/// use reqwest::blocking::Client;
	///
	/// let body = QueryRequest::make(
	/// 	["ubuntu-01".to_string(), "ubuntu-01".to_string()],
	/// 	false,
	/// 	false,
	/// 	None,
	/// 	None,
	/// 	None
	/// );
	/// let http_client = Client::new();
	/// let resp = http_client.post("...").json(&body)?.send()?.json()?;
	///
	/// ```
	pub fn make(
		query_items: &Vec<String>,
		match_regex: bool,
		show_extensions: bool,
		show_tags: bool,
		skip: Option<u64>,
		top: Option<u16>,
		subscriptions: &Option<Vec<String>>,
	) -> Self {
		let mut search_query: String = String::new();
		let mut comparison_operator: &str = "in";
		let mut extensions_join: &str = "";
		let mut tags_join: &str = "";
		let skip_param: u64 = skip.unwrap_or(0);
		let top_param: u16 = top.unwrap_or(1000);

		// ensure all vm names are lowercased
		let vm_list: Vec<String> = query_items
			.into_iter()
			.map(|vm| vm.to_lowercase())
			.collect::<Vec<String>>();

		// either interpret the query operand as a regular expression or as a list of hostname literals
		if match_regex {
			comparison_operator = "matches regex";
			search_query = format!("'{}'", vm_list[0].clone());
		} else {
			let mut query_list_iterator = vm_list.into_iter();
			search_query.push_str("(");
			search_query.push_str(
				format!(
					"'{}'",
					query_list_iterator.next().unwrap_or(String::from(""))
				)
				.as_str(),
			); // push the first one in without the preceding ', '
			while let Some(vm) = query_list_iterator.next() {
				search_query.push_str(format!(", '{}'", vm.to_lowercase()).as_str());
			}
			search_query.push_str(")");
		}

		// optionally inject join query for extension data
		if show_extensions {
			extensions_join = "| join kind=leftouter(Resources | where type =~ 'microsoft.compute/virtualmachines/extensions' | extend vmId = substring(id, 0, indexof(id, '/extensions')) | extend d = pack('name', name, 'version', properties.typeHandlerVersion) | summarize extensions = make_list(d) by vmId) on vmId";
		}

		// optionally inject tag information
		if show_tags {
			tags_join = ", tags=tags"
		}

		// template out the query
		let query = format!("Resources | where type =~ 'microsoft.compute/virtualmachines' | where tolower(tostring(name)) {} {} | extend nics=array_length(properties.networkProfile.networkInterfaces) | mv-expand nic=properties.networkProfile.networkInterfaces | where nics == 1 or nic.properties.primary =~ 'true' or isempty(nic) | project subscriptionId, rg=resourceGroup, vmId = id, vmName = name, location = tostring(location), created = tostring(properties.timeCreated), vmSize=tostring(properties.hardwareProfile.vmSize), nicId = tostring(nic.id), osType = tostring(properties.storageProfile.osDisk.osType), osName = tostring(properties.extended.instanceView.osName), osVersion = tostring(properties.extended.instanceView.osVersion), powerstate = tostring(properties.extended.instanceView.powerState.code){} {} | join kind=leftouter (ResourceContainers | where type=='microsoft.resources/subscriptions'| project sub=name, subscriptionId) on subscriptionId | join kind=leftouter (Resources| where type =~ 'microsoft.network/networkinterfaces'| extend ipConfigsCount=array_length(properties.ipConfigurations)| extend subnetId = tostring(properties.ipConfigurations[0].properties.subnet.id)| extend virtualNetwork = split(substring(subnetId, indexof(subnetId, '/virtualNetworks/') + strlen('/virtualNetworks/')), '/')[0]| extend subnet = substring(subnetId, indexof(subnetId, '/subnets/') + strlen('/subnets/'))| mv-expand ipconfig=properties.ipConfigurations| where ipConfigsCount == 1 or ipconfig.properties.primary =~ 'true'| project nicId = id, subnet, virtualNetwork, privateIp = tostring(ipconfig.properties.privateIPAddress))on nicId| order by subnet asc", comparison_operator, search_query, tags_join,extensions_join);

		Self {
			query,
			options: QueryRequestOptions::new(skip_param, None, top_param),
			subscriptions: subscriptions.to_owned(),
		}
	}
}

///
/// defines options that can be passed to a vminfo request
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryRequestOptions {
	#[serde(alias = "$skip", rename(serialize = "$skip"))]
	skip: u64,
	#[serde(alias = "$skipToken", rename(serialize = "$skipToken"))]
	skip_token: Option<String>,
	#[serde(alias = "$top", rename(serialize = "$top"))]
	top: u16,
}

impl QueryRequestOptions {
	fn new(skip: u64, skip_token: Option<String>, top: u16) -> Self {
		Self {
			skip,
			skip_token,
			top,
		}
	}
}

impl Default for QueryRequestOptions {
	fn default() -> Self {
		Self {
			skip: 0,
			skip_token: None,
			top: 1000,
		}
	}
}

///
/// special query response type for vminfo responses which can have a special format for errors thrown by Azure
///
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum QueryResponseType {
	/// defines the 200 response
	Ok(QueryResponse),
	/// defines any custom error response from Azure
	Err {
		/// entrypoint for any non-200 error body responses from Azure
		error: AzureError,
	},
}

///
/// special error type for azure Resource Graph API errors
///
#[derive(Debug, Deserialize, Serialize)]
pub struct AzureError {
	/// the Azure specific error code
	pub code: String,
	/// the Azure error message
	pub message: String,
}

///
///  Defines a format for an acceptable response from the Resource Graph API
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResponse {
	/// total number of records/results returned from the Graph API
	#[serde(alias = "totalRecords")]
	pub total_results: u64,
	/// list of Virtual Machines returned from the Graph API
	pub data: Vec<VirtualMachine>,
}

impl Default for QueryResponse {
	fn default() -> Self {
		Self {
			total_results: 0,
			data: vec![],
		}
	}
}

impl ToRedisArgs for QueryResponse {
	fn to_redis_args(&self) -> Vec<Vec<u8>> {
		let r: Vec<u8> = serde_json::to_string(self)
			.expect("cannot convert Virtual Machine to redis args")
			.as_bytes()
			.into_iter()
			.map(|i| *i)
			.collect();

		vec![r]
	}
	fn write_redis_args<W>(&self, out: &mut W)
	where
		W: ?Sized + redis::RedisWrite,
	{
		let resp: QueryResponse = self.clone();
		let resp_str =
			serde_json::to_string(&resp).expect("cannot convert Virtual Machine to redis args");

		// convert VM JSON to bytes
		let resp_bytes: &[u8] = resp_str.as_bytes();

		out.write_arg(resp_bytes)
	}
}

#[cfg(test)]
mod query_request_tests {
	#[test]
	fn single_hostname_query() {
		use super::QueryRequest;
		let hostname = vec!["linux-01".to_string()];

		let req_body = QueryRequest::make(&hostname, false, false, false, None, None, &None);

		assert_eq!(req_body.options.skip, 0);
		assert_eq!(req_body.options.top, 1000);
		assert_eq!(req_body.query.contains("matches regex"), false);
		assert!(req_body.query.contains("in ('linux-01')"));
	}

	#[test]
	fn many_hostnames_query() {
		use super::QueryRequest;
		let hostnames: Vec<String> = vec![
			"linux-01".to_string(),
			"linux-02".to_string(),
			"windows-98".to_string(),
			"ubuntu-test-04".to_string(),
		];

		let req_body = QueryRequest::make(&hostnames, false, false, false, None, None, &None);

		assert_eq!(req_body.options.skip, 0);
		assert_eq!(req_body.options.top, 1000);
		assert_eq!(req_body.query.contains("matches regex"), false);
		assert!(req_body
			.query
			.contains("in ('linux-01', 'linux-02', 'windows-98', 'ubuntu-test-04')"));
	}

	#[test]
	fn regular_expression_matching() {
		use super::QueryRequest;
		let hostnames: Vec<String> = vec!["linux-[0-9]+".to_string()];

		let req_body = QueryRequest::make(&hostnames, true, false, false, None, None, &None);

		assert_eq!(req_body.options.skip, 0);
		assert_eq!(req_body.options.top, 1000);
		assert_eq!(req_body.query.contains("matches regex"), true);
	}

	#[test]
	fn query_extensions() {
		use super::QueryRequest;
		let hostnames: Vec<String> = vec![
			"linux-01".to_string(),
			"linux-02".to_string(),
			"windows-98".to_string(),
			"ubuntu-test-04".to_string(),
		];

		let req_body = QueryRequest::make(&hostnames, false, true, false, None, None, &None);

		assert_eq!(req_body.options.skip, 0);
		assert_eq!(req_body.options.top, 1000);
		assert_eq!(req_body.query.contains("matches regex"), false);
		assert!(req_body.query.contains("linux-01"));
		assert!(req_body.query.contains("linux-02"));
		assert!(req_body.query.contains("windows-98"));
		assert!(req_body.query.contains("ubuntu-test-04"));
		assert!(req_body.query.contains("| join kind=leftouter(Resources | where type =~ 'microsoft.compute/virtualmachines/extensions'"))
	}

	#[test]
	fn query_tags() {
		use super::QueryRequest;
		let hostnames: Vec<String> = vec![
			"linux-01".to_string(),
			"linux-02".to_string(),
			"windows-98".to_string(),
			"ubuntu-test-04".to_string(),
		];

		let req_body = QueryRequest::make(&hostnames, false, false, true, None, None, &None);

		assert_eq!(req_body.options.skip, 0);
		assert_eq!(req_body.options.top, 1000);
		assert_eq!(req_body.query.contains("matches regex"), false);
		assert!(req_body.query.contains("linux-01"));
		assert!(req_body.query.contains("linux-02"));
		assert!(req_body.query.contains("windows-98"));
		assert!(req_body.query.contains("ubuntu-test-04"));
		assert!(req_body.query.contains(", tags=tags"))
	}

	#[test]
	fn query_with_custom_page_size() {
		use super::QueryRequest;
		let hostnames: Vec<String> = vec![".*linux-[0-9]+$".to_string()];

		let req_body = QueryRequest::make(&hostnames, true, false, false, None, Some(150), &None);

		assert_eq!(req_body.options.skip, 0);
		assert_eq!(req_body.options.top, 150);
	}

	#[test]
	fn query_a_page() {
		use super::QueryRequest;
		let hostnames: Vec<String> = vec![".*linux-[0-9]+$".to_string()];

		let req_body = QueryRequest::make(&hostnames, true, false, false, Some(3000), Some(1000), &None);

		assert_eq!(req_body.options.skip, 3000); // should request the 3rd page by skipping the first 3 page sizes (top)
		assert_eq!(req_body.options.top, 1000); // page size
	}
}
