[![Rust Tests](https://github.com/snltd/gurp/actions/workflows/test-rust.yml/badge.svg)](https://github.com/snltd/gurp/actions/workflows/test-rust.yml)
[![Janet Tests](https://github.com/snltd/gurp/actions/workflows/test-janet.yml/badge.svg)](https://github.com/snltd/gurp/actions/workflows/test-janet.yml)

# Gurp

Gurp is an illumos configuration management tool.

Hosts are described in a thin [Janet](https://janet-lang.org/) DSL.

```janet
(section "ntp"
         (pkg/ensure "service/network/ntpsec")

         (directory/ensure "/var/lib/ntp"
                           :owner "root"
                           :group "daemon")

         (file/ensure "/etc/ntp.conf"
                      :from "ntp.conf"
                      :label "ntp-conf")

         (svc/ensure "ntp"
                     :state "online"
                     :restarted-by [(this :file :ntp-conf)]))
```

If you want to know more:

- [An introductory walkthrough of Gurp and Janet](https://tech.id264.net/post/2025-09-01-lets-gurp).
- [Documentation for all the "doers"](doc/doers.md) - the things that do the
  things.
- [An overview of the built-in Janet helpers](doc/janet_helpers.md) that make
  life easier.
- [Examples of real configurations](https://github.com/snltd/merp/tree/main/tests/config/roles)
  taken from my own systems and used in acceptance tests.
- [A series of informal blog articles](https://tech.id264.net/tag/Gurp) which
  talk about the design, development, use, successes, shortcomings, and future
  of Gurp.

## Building and Running

Assuming you are on an illumos system with Rust, check out the repo and

```sh
$ CFLAGS=-std=c99 cargo install --path cli
```
