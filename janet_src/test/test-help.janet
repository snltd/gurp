(use judge)
(use ../lib/gurp)

(deftest test-help
  (test
    (help-for "gem")
    "\e[1m(gem/ensure)\e[0m\nNo mandatory keys\n\e[1moptional keys\e[0m\n  source     string          Source other than RubyGems. Can contain tokens and usernames\n  gem-path   string          Path to gem executable other than /opt/ooce/bin/gem\n  version    string          Gem version\n\n\e[1m(gem/remove)\e[0m\nNo mandatory keys\n\e[1moptional keys\e[0m\n  gem-path   string          Path to gem executable other than /opt/ooce/bin/gem\n  version    string          Gem version")

  (test
    (help-for "zone-fs")
    "\e[1m(zone-fs/ensure)\e[0m\n\e[1mmandatory keys\e[0m\n  special   string          The directory in the global zone\n\e[1moptional keys\e[0m\n  type      string          The type of fs mount. Default 'lofs'")

  (test
    (help-for "zone")
    "\e[1m(zone/ensure)\e[0m\n\e[1mmandatory keys\e[0m\n  brand                string          Zone brand. byhve and illumos are not \n\e[1moptional keys\e[0m\n  rctl                                 See 'zone-rctl'\n  copy-in              struct          Copy files into the zone. Key (keyword) is src, val is dest, relative to zone root. Unqualified src is assumed to be in ../files/\n  lx-image             string          Install zone using this image. See docs for pattern rules\n  final-state          string          Put the zone in the given state. Also accepts 'reboot'\n  boot-after-install   string          Boot the zone n it is installed. Default 'true'\n  clone-from           string          Instead of installing, clone from the given zone, which must exist and be halted\n  attr                                 See 'zone-attr'\n  fs                                   See 'zone-fs'\n  dns                  struct          DNS info. :domain is a string; :nameservers a tuple of strings\n  datasets             tuple           ZFS datasets (as strings) to be delegated to zone\n  net                                  See 'zone-net'\n  autoboot             string          Boot the zone on system boot. Default 'true'\n  bootstrap-from       string          Copy gurp into the zone, and apply the given file, relative to zone root\n  zonepath             string          Path to zone root\n  capped-memory        struct          Set memory cap. Keys must be :physical and :swap, values are strings like '4G'\n  exec-in              tuple           Runs the given commands (:string) in the zone after booting\n  recreate             number          1-in-n chance the zone will be destroyed and recreated. Default '0'\n\n\e[1m(zone/remove)\e[0m\nNo mandatory keys\nNo optional keys")

  (test
    (help-for "pkg")
    "\e[1m(pkg/ensure)\e[0m\nNo mandatory keys\nNo optional keys\n\n\e[1m(pkg/remove)\e[0m\nNo mandatory keys\nNo optional keys")

  (test
    (help-for "cron")
    "\e[1m(cron/ensure)\e[0m\n\e[1mmandatory keys\e[0m\n  command         string          Command which runs\n\e[1moptional keys\e[0m\n  minute          string|number   Minute(s) job runs at. Accepts divisions and ranges. Default '*'\n  day-of-week     string|number   Numeric day(s) on  which job runs. 0=Sunday. Default '*'\n  hour            string|number   Hour(s) at which job runs. Default '*'\n  month-of-year   string|number   Month(s) in which job runs. Default '*'\n  user            string          Username which runs job. Must already exist. Default 'root'\n  day-of-month    string|number   Day(s) of month on which job runs. Default '*'\n\n\e[1m(cron/remove)\e[0m\nNo mandatory keys\nNo optional keys")

  (test
    (help-for "directory")
    "\e[1m(directory/ensure)\e[0m\nNo mandatory keys\n\e[1moptional keys\e[0m\n  owner   string|number   The username or UID of the user who owns this directory. Default 'root'\n  group   string|number   The group name or GID of the for this directory. Default 'root'\n  mode    string          Permissions written as a four-digit octal. Default '0755'"))

(deftest test-help-missing
  (test (help-for "WAT?") "No help for 'WAT?'"))
