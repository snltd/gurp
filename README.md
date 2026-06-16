[![Rust Tests](https://github.com/snltd/gurp/actions/workflows/test-rust.yml/badge.svg)](https://github.com/snltd/gurp/actions/workflows/test-rust.yml)
[![Janet Tests](https://github.com/snltd/gurp/actions/workflows/test-janet.yml/badge.svg)](https://github.com/snltd/gurp/actions/workflows/test-janet.yml)
[![illumos Build](https://github.com/snltd/gurp/actions/workflows/release.yml/badge.svg)](https://github.com/snltd/gurp/actions/workflows/release.yml&nocache=1)

# Gurp

Gurp is an illumos configuration management tool.

## Features

- Configuration is described in a simple but powerful Lisp DSL
- illumos features like Zones, SMF, ZFS, Crossbow etc are first-class citizens
- Single-binary deployment for client and server
- Client/server mode, or standalone
- Fast - client runs usually take under a second
- Efficient - client and server typically use ~20Mb RSS
- Client and server OTEL telemetry
- No YAML

## Example

```janet
(import ../globals)

(def smf-method-path (pathcat globals/site-smf-method "minidlna.sh"))
(def smf-service "sysdef/multimedia/minidlna")

(def minidlna-conf
  [:media_dir "A,/storage/flac"
   :friendly_name (fact :hostname)
   :album_art_names "front.jpg"
   :strict_dlna "no"
   :notify_interval 900])

(host "minidlna-server"
      (pkg/ensure "ooce/multimedia/minidlna")

      (file/ensure smf-method-path
                   :mode "0755"
                   :from "minidlna/minidlna-method.sh")

      (file/ensure "/etc/opt/ooce/minidlna/minidlna.conf"
                   :from-struct minidlna-conf
                   :to-format "k=v")

      (svc/ensure smf-service :state "online")

      (smf/ensure smf-service
                  :fmri smf-service
                  :description "MiniDLNA - DLNA/UPnP-AV media server"
                  (smf/method "start"
                              :exec smf-method-path
                              :user "minidlna"
                              :group "minidlna")
                  (smf/method "refresh"
                              :exec smf-method-path
                              :user "minidlna"
                              :group "minidlna")))
```

If you want to know more:

- [An introductory walkthrough of Gurp and Janet](https://tech.id264.net/post/2025-09-01-lets-gurp).
- [Documentation for all the "doers"](doc/doers) - the things that do the
  things.
- [An overview of the built-in Janet DSL](doc/dsl.md) that make life easier.
- [Examples of real configurations](https://github.com/snltd/merp/tree/main/tests/config/roles)
  taken from my own systems and used in acceptance tests.
- [A series of informal blog articles](https://tech.id264.net/tag/Gurp) which
  talk about the design, development, use, successes, shortcomings, and future
  of Gurp.

## Get Gurp

- [Download an OmniOS binary](https://github.com/snltd/gurp/releases/).

- Build it yourself:
  ```sh
  $ git clone git@github.com:snltd/gurp.git && cd gurp
  $ CFLAGS=-std=c99 cargo install --path cli
  ```

- [Get an omnios-extra style build dir](https://github.com/snltd/sysdef-extra/tree/main/build/gurp).
