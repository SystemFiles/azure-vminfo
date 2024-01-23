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
A simple utility written in Rust to pull useful virtual machine info from a configured Azure tenant using the Azure Resource Graph APIs

Usage: vminfo [OPTIONS] [vm_name_or_regexp]...

Arguments:
  [vm_name_or_regexp]...  Specifies one or more VM name(s) or a regular expression to match VM(s)

```
> Use `--help` to get a full list of options that can be used

## Maintainer(s) / Contributor(s)

- Ben Sykes <ben.sykes@statcan.gc.ca>