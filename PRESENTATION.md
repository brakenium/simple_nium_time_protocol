% SNTP server in Rust
% Bram van Kempen
% 2023-10-07

# What did I do?

- SNTP server
- Rust
- [RFC 4330](https://tools.ietf.org/html/rfc4330)
- [https://github.com/brakenium/simple_nium_time_protocol/](https://github.com/brakenium/simple_nium_time_protocol/)

::: notes

I implemented a simple SNTP server in Rust. It is based on [RFC 4330](https://tools.ietf.org/html/rfc4330).
SNTP is a simplified version of NTP, which is a protocol for synchronizing clocks over a network.
Time tends to be inherently trusted in a lot of applications, so knowing more about the inner workings is a good
thing. I also wanted to learn some new things which I will talk about later.

:::

------------------

# My goals

- I wanted to learn how to implement an RFC
- Do a more "serious" and larger project
- Further my knowledge of Rust
- Learn how to use UDP or TCP sockets
- Learn how to implement a program without using libraries for the core goal

------------------

# What is Rust?

- Low level
  - Used in Windows, Linux kernel and Firefox
- Compiled
  - Microcontroller?

::: notes

Rust is a programming language that focuses on safety and performance. It is used in a lot of places, but most notably
in Firefox and the Linux kernel. Microsoft is now also using Rust in some parts of Window's core.
Rust can be compiled and run without a runtime on most platforms. You can probably get this server to run on a
microcontroller if you wanted to.

:::

------------------

# Current state

- Usable by windows
- CLI tools can request
- Time parsing bugs
- Not strictly RFC compliant
- Simple to modify
  - Pentesting?

::: notes

- Windows will accept this server as a timeserver
- Command line tools can query the server
- There are some bugs related to time parsing
- The server is not fully compliant with the RFC
- The server is fairly simple and modifying responses can be done easily
  - Might be useful in certain pentesting scenarios

:::

------------------

# An NTP packet

```rust
pub struct NtpMessage {
  // Warn of leap seconds to be inserted or deleted in the last minute of the current day
  pub li: LeapIndicator,
  // NTP version
  pub vn: VersionNumber,
  // Mode of operation: client, server, broadcast, etc.
  pub mode: Mode,
  // For the server to notify to the client the following:
  // If it is synchronised with a high accuracy clock, radio receiver, GPS, etc.
  // If it is synchronised via NTP or SNTP
  // If the client should stop sending requests
  pub stratum: Stratum,
  // Tells the client how long it can wait before a new request. From 16s to 36h
  pub poll_interval: u8,
  // How precise the server's clock is
  pub precision: i8,
  // Total time from the server to its reference clock
  pub root_delay: i32,
  // How precise the resulting time is
  pub root_dispersion: u32,
  // Where the server gets its time from
  pub reference_identifier: Option<ReferenceIdentifier>,
  // When the system's clock was last updated
  pub reference_timestamp: Option<NtpTimestamp>,
  // Time at which client sent its request
  pub originate_timestamp: Option<NtpTimestamp>,
  // Time at which the server received the request
  pub receive_timestamp: Option<NtpTimestamp>,
  // Time at which the packet was sent
  pub transmit_timestamp: NtpTimestamp,
}
```

------------------

# Time parsing

## The standards
- 64 bit NTP timestamp
  - 32 bit seconds
  - 32 bit fraction
  - Since 1900-01-01 00:00:00 UTC
- Unix timestamp
  - Since 1970-01-01 00:00:00 UTC

------------------

# Time parsing

## The bug

- Seconds are correct
- Fraction is not
- 0-padding?

::: notes

SNTP uses the number of seconds since 1900-01-01 00:00:00 UTC as a timestamp. This is a 64-bit integer consisting of
a 32-bit seconds part and a 32-bit fraction part. In most situations people use Unix time, which is the number of
seconds since 1970-01-01 00:00:00 UTC. This project uses an existing library to store time in the Unix format with some
manual parsing to and from NTP timestamps.

Somewhere in the conversion between the formats there is a bug. Most probably with the fraction part. I have not found
the exact cause yet. However, when writing this I found some information I missed before. The RFC specifies that the
fraction has some bits that are always set to 0. This might be the cause of the bug. In here is also where some
security improvements might be made by setting these bits to a value other than 0.

:::

------------------

# Demo's

## Demo 1

Query the server with the `sntp` and `w32tm` command line tools. Inspect the resulting packets in Wireshark.

------------------

# Demo's

## Demo 2

Set the Windows timeserver to this server and show that the time is synchronized.

------------------

# Demo's

## Demo 3

Adjust some return values and show how this affects the time on the client while inspecting the packets in Wireshark.

------------------

# Possible improvements

- Implement the full RFC
- Fix the time parsing bug
- Implement the suggested NTP timestamp security improvements

------------------

# Possible improvements

- Add some way to modify the response without changing the code
  - Simple things like doing math operations on the time
- Use threads instead of async/await
  - This might be a good way to learn more about threads in Rust
  - Significantly easier to understand and improve performance

------------------

# What did I learn?

## Original goals

- I wanted to learn how to implement an RFC ✔️
- Do a more "serious" and larger project ✔️
  - serious ✔️
  - Larger ❌
- Further my knowledge of Rust ✔️
- Learn how to use UDP or TCP sockets ✔️
- Learn how to implement a program without using libraries for the core goal ✔️

------------------

# What did I learn?

## Unexpected

- Extracting multiple values from a single byte ✔️
- Wireshark is very useful for debugging since it correctly parses the packets ✔️

------------------

# Wrap-Up

Source code: [https://github.com/brakenium/simple_nium_time_protocol/](https://github.com/brakenium/simple_nium_time_protocol/)

- Server and responses only 100 lines of code
- Packet parsing is 445 lines of code
- Presentation as markdown and html
