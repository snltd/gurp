[![Rust Tests](https://github.com/snltd/gurp/actions/workflows/test-rust.yml/badge.svg)](https://github.com/snltd/gurp/actions/workflows/test-rust.yml)
[![Janet Tests](https://github.com/snltd/gurp/actions/workflows/test-janet.yml/badge.svg)](https://github.com/snltd/gurp/actions/workflows/test-janet.yml)

# Gurp

Gurp is an [illumos](https://www.illumos.org/) configuration management tool.

## Features

- Configuration is described in a simple but powerful
  [Lisp-like](https://janet-lang.org/) DSL.
- illumos features like Zones, SMF, ZFS, Crossbow etc are first-class citizens.
- Client/server mode, or standalone.
- Single-binary deployment for client and server.
- Fast -- client runs usually take under a second.
- Efficient -- client and server typically use ~20Mb RSS.
- Client and server OTEL metrics and logs.
- No YAML.

## Example

```janet
(import ../globals) # for common config

(def smf-method-path (pathcat globals/site-smf-method "minidlna.sh"))
(def smf-service "sysdef/multimedia/minidlna")

# Define application config as a structure, and trust Gurp to turn it into a
# file of the appropriate kind.
(def minidlna-conf
  [:media_dir "A,/storage/flac"
   :friendly_name (fact :hostname)
   :album_art_names "front.jpg"
   :strict_dlna "no"
   :notify_interval 900])

# Hosts can be made up of bare resources, or resources can be bundled into
# roles for composition.
(host "minidlna-server"
      (pkg/ensure "ooce/multimedia/minidlna")

      # Copy in files stored alongside configuration.
      (file/ensure smf-method-path
                   :mode "0755"
                   :from "minidlna/minidlna-method.sh")

      # Or generate files from data.
      (file/ensure "/etc/opt/ooce/minidlna/minidlna.conf"
                   :from-struct minidlna-conf
                   :to-format "k=v")

      (svc/ensure smf-service :state "online")

      # Even SMF manifests can be defined programmatically.
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
- [What's new in v2.x](https://tech.id264.net/post/2026-05-03-gurp-v2).

## Get Gurp

- [Download an OmniOS binary](https://github.com/snltd/gurp/releases/).

- Build it yourself:
  ```sh
  $ git clone git@github.com:snltd/gurp.git && cd gurp
  $ CFLAGS=-std=c99 cargo install --path cli
  ```

- [Get an omnios-extra style build dir](https://github.com/snltd/sysdef-extra/tree/main/build/gurp).
