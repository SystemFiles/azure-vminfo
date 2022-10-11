# azure-vminfo
A simple utility written in Rust to pull useful virtual machine info from a configured Azure tenant using the Azure Resource Graph APIs

## Installation

### Install with Cargo

```bash
cargo install azure-vminfo
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

Options:
      --login         Specifies whether to prompt for credentials manually (will exit)
  -r, --match-regexp  Specifies whether or not to enable regexp matching
  -e, --extensions    Specifies whether or not to display Azure extensions for each VM
  -h, --help          Print help information
```