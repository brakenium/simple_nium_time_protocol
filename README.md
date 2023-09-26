# Simple Nium Time Protocol (SNTP) server

This is a simple SNTP server written in Rust. It is (loosely) based on [RFC4330](https://datatracker.ietf.org/doc/html/rfc4330#section-3)
Full compatibility to the spec is not guaranteed. This is especially the case for the defined server operations.
The Windows 10 NTP client can use this server just finegh
.

## Prerequisites

This project uses the Rust programming language. You will need to install the tooling using Rustup
which can be installed from [rustup.rs](https://rustup.rs/) or your system package manager.
The Rust version used to run the program is `1.72.0`. Any version before might work fine too. Any version 
later will more than likely still work.

## Usage

Windows reserves the default NTP port 123 for the client source port. You will need to run
the NTP server on another machine if you are on a Windows machine. A Linux system will work
with Sudo privileges.

Run server with debug output. Useful for testing the code.
```bash
cargo b && sudo ./target/debug/simple_nium_time_protocol
```

Run server in release mode. Useful in case better performance is necessary. There are other compiler
options for an even bigger improvement.
```bash
cargo b --release && sudo ./target/release/simple_nium_time_protocol
```

## Useful Windows time commands

Query the time-server and display dispersion and offset time
```cmd
w32tm /stripchart /computer:172.26.131.30
```

Set upstream time server
```cmd
w32tm /config /manualpeerlist:<server IP> /syncfromflags:manual /update
```

Show current configuration of localhost
```cmd
w32tm /query /computer:localhost /configuration
```
