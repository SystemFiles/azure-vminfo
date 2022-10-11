
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).



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
- `60bced0` - ICP-9510 Test Build (16)
- `53132d0` - ICP-9510 Test Build (15)
- `60cea05` - ICP-9510 Test Build (14)
- `fce0f39` - ICP-9510 Test Build (13)
- `2e720be` - ICP-9510 added remainder for final page
- `1ad2af3` - ICP-9510 Test Build (12)
- `0ed2eff` - ICP-9510 Test Build (11)
- `2e8b565` - ICP-9510 Test Build (10)
- `bb2a001` - ICP-9510 Test Build (9)
- `739c165` - ICP-9510 Test Build (8)
- `9065081` - ICP-9510 Test Build (7)
- `c5ffc6f` - ICP-9510 Test Build (6)
- `52bc364` - ICP-9510 Test Build (5)
- `682293b` - ICP-9510 made Virtual Machine data safer
- `f86b846` - ICP-9510 Test Build (4)
- `4e620eb` - ICP-9510 custom deserializer implemented for ip addresses
- `324cf7a` - ICP-9510 made ipv4 optionally nullable
- `0cf1045` - ICP-9510 handle null|string in serialization/deserialization
- `d99944b` - ICP-9510 fixed field name for vmName
- `64c1cb8` - ICP-9510 implemented remaining fields for Virtual Machines
- `c391568` - ICP-9510 correct app config name in primary load
- `2ec4bc5` - ICP-9510 implemented credential query and save to config
- `c8cbc82` - ICP-9510 removed --dry-run from cli args in crate deploy CI
- `1fd9ae1` - ICP-9510 CI/CD implemented deploy for libs and CLI util
- `2679b3e` - ICP-9510 Test Build (3)
- `66fd53f` - ICP-9510 Test Build (2)
- `dd7fc14` - ICP-9510 Test Build (1)
- `54ac623` - ICP-9510 Test Build (0)
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
