
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [ 1.2.0 ] - 2024-01-23

> Feat(cli): added additional search fields

### Added

- feat(cli): added additional search field for tags

## [ 1.1.1 ] - 2022-11-9 15:13:41

> BENS-0012 Hotfix: Resolve Strange Behaviour of Caching with Specific Keys

### Changed

- `30e9334` - BENS-0012 updated crate versions
- `3975eb2` - BENS-0012 some better name handling for vm caching
- `cba5759` - BENS-0012 added some debugging



## [ 1.1.0 ] - 2022-11-8 23:0:49

> CLOUD-16508 Resource Result Caching

### Added

- `a897470` - CLOUD-16508 fixed up some of the documentation of CLI options
- `9985bef` - CLOUD-16508 added no-cache option to cli
- `ef527fb` - CLOUD-16508 squashed bug where cached results dont come through with all bad non-cached queries
- `38eac85` - CLOUD-16508 removed caching as separate compile feature since client supports no-cache out of the box
- `d58b1db` - CLOUD-16508 removed debug messages
- `e862684` - CLOUD-16508 added very basic cache retrieval for query method on VMInfo client
- `5954b59` - CLOUD-16508 added very basic cache retrieval for query method on VMInfo client



## [ 1.0.4 ] - 2022-11-3 18:56:24

> BENS-0011 hotfix: resolved dependency for cross-compile linux and macos

### Changed

- `770652b` - BENS-0011 fix applied



## [ 1.0.3 ] - 2022-11-3 18:18:38

> BENS-0010 Hotfix for doctests

### Changed

- `ed3da04` - BENS-0010 added ignore for invalid doctests



## [ 1.0.2 ] - 2022-11-3 18:12:30

> BENS-0009 cleanup for lib and better documentation

### Changed

- `02d4e75` - BENS-0009 cleaned up and refactored token persistance for lib vminfo
- `5b7c477` - BENS-0009 checkpoint



## [ 1.0.1 ] - 2022-11-2 22:59:28

> BENS-0008 better refresh handling

### Changed

- `f2e9acb` - BENS-0008 final fixup
- `a8d3d22` - BENS-0008 removed old debug messages
- `0315f11` - BENS-0008 missing logout option description
- `a4dffc3` - BENS-0008 added option to clear local token cache
- `71f8d99` - BENS-0008 refactored auth again



## [ 1.0.0 ] - 2022-11-1 21:37:40

> ICP-9762-d1 Interactive Device-code OAuth2.0 Flow for User Auth

### Added

- `adc5296` - ICP-9762-d1 updated feature meta



## [ 0.0.9 ] - 2022-10-17 14:14:36

> BENS-0007 bump crate versions

### Changed

- `20d24b4` - BENS-0007 bumped lib and cli



## [ 0.0.8 ] - 2022-10-17 14:11:32

> BENS-0006 hotfix: ensure all cases supported for vm and regexp operands

### Changed

- `aa6ea7c` - BENS-0006 added lowecase formatter to query body make function



## [ 0.0.7 ] - 2022-10-13 19:32:49

> BENS-0005 Small Changes for Text

### Changed

- `a7e69a7` - BENS-0005 updated license and cli description
- `eed9e1e` - BENS-0005 fix license for cargo



## [ 0.0.6 ] - 2022-10-12 21:17:1

> BENS-0005 fix license for cargo

### Changed

- `f76206c` - BENS-0005 license fix applied to lib



## [ 0.0.5 ] - 2022-10-12 21:4:54

> BENS-0004 update crate metadata and refactor AzCreds from CLI util mod

### Changed

- `10f007c` - BENS-0004 more descriptive cargo metadata and refactored some of the credential handling code



## [ 0.0.4 ] - 2022-10-12 0:2:33

> BENS-0003 hotfix: added native dependencies to prerequisites and fixed a typo

### Changed

- `3be8e0c` - BENS-0003 native deps + typo fix + pretty output re-enabled



## [ 0.0.3 ] - 2022-10-11 23:43:31

> BENS-0002 hotfix: bump lib ver

### Changed

- `01d9c80` - BENS-0002 added initial README for lib-vminfo
- `508c69d` - BENS-0002 bumped cargo lib ver



## [ 0.0.2 ] - 2022-10-11 23:27:50

> BENS-0001 Hotfix to fix dependencies so that crates.io likes us

### Changed

- `4a23684` - BENS-0001 updated dependency list with no wildcards and added rust install to docs



## [ 0.0.1 ] - 2022-10-11 23:10:54

> ICP-9510 Implement vminfo in rust with paging

### Changed

- `60ee8a6` - ICP-9510 added some basic usage and installation docs to the README
- `ccc524f` - ICP-9510 remove color since it mysteriously resorts keys in an ugly way (will revisit)
- `4eae6b3` - ICP-9510 added color to output
- `d96b626` - ICP-9510 remove old debug code
- `2e720be` - ICP-9510 added remainder for final page
- `682293b` - ICP-9510 made Virtual Machine data safer
- `4e620eb` - ICP-9510 custom deserializer implemented for ip addresses
- `324cf7a` - ICP-9510 made ipv4 optionally nullable
- `0cf1045` - ICP-9510 handle null|string in serialization/deserialization
- `d99944b` - ICP-9510 fixed field name for vmName
- `64c1cb8` - ICP-9510 implemented remaining fields for Virtual Machines
- `c391568` - ICP-9510 correct app config name in primary load
- `2ec4bc5` - ICP-9510 implemented credential query and save to config
- `c8cbc82` - ICP-9510 removed --dry-run from cli args in crate deploy CI
- `1fd9ae1` - ICP-9510 CI/CD implemented deploy for libs and CLI util
- `c048950` - ICP-9510 added deployment workflow with dry-run for now
- `bb2de15` - ICP-9510 added tests
- `b697c32` - ICP-9510 fixed regexp match query formatting
- `3464275` - ICP-9510 refactored into separate lib crate
- `bc44929` - ICP-9510 moved access token to its own auth module
- `3283823` - ICP-9510 implemented basic paging algorithm for vminfo results (slow)
- `bed8a41` - ICP-9510 ready for testing in enterprise tenant
- `d66dee9` - ICP-9510 a little more robust error handling framework
- `48a4a81` - ICP-9510 broke some stuff for the sake of better error handling
- `cdadc77` - ICP-9510 api somewhat implemented ... still needs work
- `525ab29` - ICP-9510 build some more supporting code
- `de3373d` - ICP-9510 got something working