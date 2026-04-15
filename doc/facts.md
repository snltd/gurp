# Facts

Unlike some similar tools, Gurp does not offer many "facts". This is because, as
you have the power of a real programming language, it is generally easy to find
things out as you need to. e.g. `(os/cpu-count)` or `(slurp /etc/resolv.conf)`.

However, there is a `fact` function which you can use in your config, which
provides a few points of information which are commonly used and/or are slightly
cumbersome to come by.

Here it is in the REPL.

```
$ gurp repl
repl:1:> (fact :hostname)
"serv"
repl:2:> (fact :zonename)
"global"
repl:3:> (fact :ip-addresses)
{"e1000g0/v4" {:addr "192.168.1.5/24" :state "ok" :type "static"}}
repl:4:> (keys (fact :zones))
@["merp-ngz-doer" "serv-merp" "serv-ws" "serv-records" "serv-fs" "serv-backup" "serv-grafana" "global" "serv-build" "serv-proxy" "merp-gold-zone" "serv-gurp" "serv-metrics" "serv-pkg" "serv-media" "illumos-test" "serv-dns" "lipkg-green" "serv-cron" "serv-mariadb" "lipkg-blue"]
repl:4:> (fact :uname)
{:bustype "<unknown>" :kernelid "omnios-r151056-1acbca4f5bd" :machine "i86pc" :node "serv" :numcpu 4 :oem# 0 :origin# 1 :release 5.11 :serial "<unknown>" :system "SunOS" :users "<unknown>"}
repl:5:> ((fact :uname) :kernelid)
"omnios-r151056-1acbca4f5bd"
```

Current facts are:

- `:hostname` (string) the output of `uname -n`.
- `:zonename` (string) the output of `zonename`.
- `:zones` (struct) a struct of the output of `zoneadm -cv`, keyed on zone name.
- `:links` (struct) a struct of the output of `dladm show-link`, keyed on link
  name,
- `:ip-addresses` (struct) a struct of the output of `ipadm show-addr`, keyed on
  address name,
- `:ip-interfaces` (struct) a struct of the output of `ipadm show-if`, keyed on
  address name,

Facts are cached on each Gurp run. To avoid the cache, pass `true` as a second
argument to `fact`.
