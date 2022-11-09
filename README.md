# azure-vminfo
A Rust utility that pulls useful virtual machine metadata and instance data from a configured Azure tenant using the Azure Resource Graph API(s)

## Installation

### Prerequisites

The instructions to install assume you have the rust toolkit installed. You can install them using `rustup`

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**note:** you will also need `openssl-devel` on RHEL or `libssl-devel` on Ubuntu/Fedora

### Install with Cargo

```bash
cargo install az-vminfo
```

### Install from Source

```bash
# clone the project
git clone https://github.com/SystemFiles/azure-vminfo.git && cd ./azure-vminfo

# install
cargo install --path .
```

## Usage

```
A Rust utility to pull useful virtual machine data from a configured Azure tenant using the Azure Resource Graph APIs

Usage: vminfo [OPTIONS] [vm_name_or_regexp]...

Arguments:
  [vm_name_or_regexp]...  Specifies one or more VM name(s) or a regular expression to match VM(s)

Options:
      --login              Specifies whether to prompt for credentials manually (will exit). Will default to user authentication method
      --logout             Perform full logout operation. This will clear the credential/token cache and remove the user from the system
      --service-principal  Specifies that azure-vminfo should use a service-principal (client_id and client_secret) to authenticate
      --interactive        Specifies that azure-vminfo should use an interactive (client_id and login challenge) authentication method
  -c, --no-cache           Specifies whether to ignore the cache and force data to be pulled from Resource Graph API directly
  -r, --match-regexp       Specifies whether or not to enable regexp matching
  -e, --extensions         Specifies whether or not to display Azure extensions for each VM
  -h, --help               Print help information
```

## Maintainer(s) / Contributor(s)

- Ben Sykes <ben.sykes@statcan.gc.ca>