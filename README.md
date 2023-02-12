# check_mailcow
A nagios-like binary that runs various checks on self-hosted [Mailcow](https://github.com/mailcow/mailcow-dockerized) installations.

# Checks

* Queue - Checks to ensure the outbound message queue is empty
* Quotas - Checks to ensure all mailboxes are not in danger of exceeding their quota

# Usage
Basic usage:

```
#!/bin/sh

export MAILCOW_API_KEY=<api key>
./check_mailcow
```

The binary will exit with errors and a non-zero code if any checks fail.

See `check_mailcow --help` for more usage information.

# Building
1. [Install Rust](https://www.rust-lang.org/tools/install)
1. [Install Docker](https://docs.docker.com/engine/install/)
1. Install `cross`: `cargo install -f cross`
1. `./buildall.sh`

# TODO

* Test suite
* GitHub publishing pipeline