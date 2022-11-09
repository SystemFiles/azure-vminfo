# lib-vminfo

## About

A small library designed to make querying detailed VM information from Azure Resource Graph as simple and painless as possible

## Installation

To install and use this library, simply add it to your `[dependencies]` in your `Cargo.toml`

```toml
[dependencies]
lib_vminfo = { version = "1.0", path = "./lib_vminfo" }
```

## Usage

```rust

// get the first 100 VMs that match the provided regexp
let resp: QueryResponse = client.query_vminfo(
	vec!["ubuntu-vm[0-9]+"],
	true,
	false,
	Some(0),
	Some(100),
)?;

...
```

## License

MIT License

Copyright (c) His Majesty the King in Right of Canada, as represented by the minister responsible for Statistics Canada, 2022.

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

## Maintainer(s)

- Ben Sykes (ben.sykes@statcan.gc.ca)

