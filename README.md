# `nu_plugin_ulid`

A [Nushell](https://www.nushell.sh/) plugin that adds support for ULIDs.

## Usage

After this plugin has been [installed](https://www.nushell.sh/book/plugins.html) (it is available
from [crates.io](https://crates.io/crates/nu_plugin_ulid)), you can access the following commands:

### `random ulid`

```
Generate a random ulid

Usage:
  > random ulid {flags} 

Flags:
  -h, --help: Display the help message for this command
  -0, --zeroed: Fill the random portion of the ulid with zeros (incompatible with --oned)
  -1, --oned: Fill the random portion of the ulid with ones (incompatible with --zeroed)

Input/output types:
  ╭───┬─────────────────────────────────────────────┬────────╮
  │ # │                    input                    │ output │
  ├───┼─────────────────────────────────────────────┼────────┤
  │ 0 │ nothing                                     │ string │
  │ 1 │ datetime                                    │ string │
  │ 2 │ record<timestamp: datetime, random: string> │ string │
  │ 3 │ record<timestamp: datetime, random: int>    │ string │
  │ 4 │ record<timestamp: datetime>                 │ string │
  │ 5 │ record<random: string>                      │ string │
  │ 6 │ record<random: int>                         │ string │
  ╰───┴─────────────────────────────────────────────┴────────╯

Examples:
  Generate a random ulid based on the current time
  > random ulid
  01KAXYA0XEPGFK402HNMA1ZR5P

  Generate a random ulid based on the given timestamp
  > 2024-03-19T11:46:00 | random ulid
  01HSB8GP60B6SQMEH550PTPKZN

  Generate a ulid based on the current time with the random portion all set to 0 (useful when sorting or comparing ULIDs)
  > random ulid --zeroed
  01KAXYA0XE0000000000000000
```

### `parse ulid`

```
Parse a ulid into a date

Usage:
  > parse ulid 

Flags:
  -h, --help: Display the help message for this command

Input/output types:
  ╭───┬────────┬─────────────────────────────────────────────╮
  │ # │ input  │                   output                    │
  ├───┼────────┼─────────────────────────────────────────────┤
  │ 0 │ string │ record<timestamp: datetime, random: string> │
  ╰───┴────────┴─────────────────────────────────────────────╯

Examples:
  Generate a ulid and parse out the date portion
  > random ulid | parse ulid | get timestamp
  Tue, 25 Nov 2025 16:42:19 +0000 (35 minutes ago)
```

## Parsed Format

This plugin uses a specific format for the parsed representation of a ULID in order to make it useful
and to prevent round-trip data loss:

```rust
struct Ulid {
    timestamp: DateTime,
    random: String,
}
```

While `random ulid` can parse int `random` fields, a string is used when outputting the components
as ULIDs use a 80-bit number for the random portion (and are 128-bits in total), and Nushell currently
[does not support ints larget than `i64`](https://github.com/nushell/nushell/issues/8054).
When/if Nushell supports either 128-bit numbers or bigints in general this format will be updated
to output that, however it will always support parsing strings since I have no reason not to.
