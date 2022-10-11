use super::vm::VirtualMachine;
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
	/// let body = QueryRequest::make(["l-sykeben-1", "l-sykeben-2", "l-sykeben-3", "l-sykeben-N"]);
	/// let http_client = Client::new();
	/// let resp = http_client.post("...").json(&body)?.send()?.json()?;
	///
	/// ```
	pub fn make(
		vm_query_list: &Vec<String>,
		match_regex: bool,
		show_extensions: bool,
		skip: Option<u64>,
		top: Option<u16>,
		subscriptions: &Option<Vec<String>>,
	) -> Self {
		let mut search_query: String = String::new();
		let mut comparison_operator: &str = "in";
		let mut extensions_join: &str = "";
		let skip_param: u64 = skip.unwrap_or(0);
		let top_param: u16 = top.unwrap_or(1000);

		// either interpret the query operand as a regular expression or as a list of hostname literals
		if match_regex {
			comparison_operator = "matches regex";
			search_query = format!("'{}'", vm_query_list[0].clone());
		} else {
			let mut query_list_iterator = vm_query_list.into_iter();
			search_query.push_str("(");
			search_query.push_str(
				format!(
					"'{}'",
					query_list_iterator.next().unwrap_or(&String::from(""))
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

		// template out the query
		let query = format!("Resources | where type =~ 'microsoft.compute/virtualmachines' | where tolower(tostring(name)) {} {} | extend nics=array_length(properties.networkProfile.networkInterfaces) | mv-expand nic=properties.networkProfile.networkInterfaces | where nics == 1 or nic.properties.primary =~ 'true' or isempty(nic) | project subscriptionId, rg=resourceGroup, vmId = id, vmName = name, location = tostring(location), created = tostring(properties.timeCreated), vmSize=tostring(properties.hardwareProfile.vmSize), nicId = tostring(nic.id), osType = tostring(properties.storageProfile.osDisk.osType), osName = tostring(properties.extended.instanceView.osName), osVersion = tostring(properties.extended.instanceView.osVersion), powerstate = tostring(properties.extended.instanceView.powerState.code) {} | join kind=leftouter (ResourceContainers | where type=='microsoft.resources/subscriptions'| project sub=name, subscriptionId) on subscriptionId | join kind=leftouter (Resources| where type =~ 'microsoft.network/networkinterfaces'| extend ipConfigsCount=array_length(properties.ipConfigurations)| extend subnetId = tostring(properties.ipConfigurations[0].properties.subnet.id)| extend virtualNetwork = split(substring(subnetId, indexof(subnetId, '/virtualNetworks/') + strlen('/virtualNetworks/')), '/')[0]| extend subnet = substring(subnetId, indexof(subnetId, '/subnets/') + strlen('/subnets/'))| mv-expand ipconfig=properties.ipConfigurations| where ipConfigsCount == 1 or ipconfig.properties.primary =~ 'true'| project nicId = id, subnet, virtualNetwork, privateIp = tostring(ipconfig.properties.privateIPAddress))on nicId| order by subnet asc", comparison_operator, search_query, extensions_join);

		Self {
			query,
			options: QueryRequestOptions::new(skip_param, None, top_param),
			subscriptions: subscriptions.to_owned(),
		}
	}
}

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

/// Defines a format for an acceptable response from the Resource Graph API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResponse {
	#[serde(alias = "totalRecords")]
	pub total_results: u64,
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

#[cfg(test)]
mod query_request_tests {
	#[test]
	fn single_hostname_query() {
		use super::QueryRequest;
		let hostname = vec!["linux-01".to_string()];

		let req_body = QueryRequest::make(&hostname, false, false, None, None, &None);

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

		let req_body = QueryRequest::make(&hostnames, false, false, None, None, &None);

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

		let req_body = QueryRequest::make(&hostnames, true, false, None, None, &None);

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

		let req_body = QueryRequest::make(&hostnames, false, true, None, None, &None);

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
	fn query_with_custom_page_size() {
		use super::QueryRequest;
		let hostnames: Vec<String> = vec![".*linux-[0-9]+$".to_string()];

		let req_body = QueryRequest::make(&hostnames, true, false, None, Some(150), &None);

		assert_eq!(req_body.options.skip, 0);
		assert_eq!(req_body.options.top, 150);
	}

	#[test]
	fn query_a_page() {
		use super::QueryRequest;
		let hostnames: Vec<String> = vec![".*linux-[0-9]+$".to_string()];

		let req_body = QueryRequest::make(&hostnames, true, false, Some(3000), Some(1000), &None);

		assert_eq!(req_body.options.skip, 3000); // should request the 3rd page by skipping the first 3 page sizes (top)
		assert_eq!(req_body.options.top, 1000); // page size
	}
}
