(use judge)
(use ../src/dsl)

(deftest labelise
  (test (labelise "/some/file" 1 2 3) "_some_file-1-2-3")
  (test (labelise :key1 :key2 :key3) "key1-key2-key3")
  (test (labelise "string" 123 :keyword) "string-123-keyword"))

(deftest this
  (setdyn :role-dyn (string (quote basenode)))
  (test (this "file" "the-label" "owner") :/basenode/file/the-label/owner)
  (test (this "file" "the-label") :/basenode/file/the-label)
  (setdyn :role-dyn nil))

(deftest pathcat
  (def var1 "/opt/site")
  (def var2 "lib")
  (test (pathcat var1 "/chunk-a" var2 "chunk-b" "file.tar")
        "/opt/site/chunk-a/lib/chunk-b/file.tar")
  (test (pathcat "/opt/site/chunk-a/lib/chunk-b/file.tar")
        "/opt/site/chunk-a/lib/chunk-b/file.tar"))

(deftest zfscat
  (def big-pool "big")
  (test (zfscat big-pool "export" "flac") "big/export/flac")
  (test (zfscat big-pool "") "big"))

(deftest argcat
  (test (argcat "/bin/cat" "file1" "file2") "/bin/cat file1 file2")
  (test (argcat "judge" "test.janet") "judge test.janet"))

(deftest fields
  (test
    (fields "f1 f2 f3 f4    f5 ") @["f1" "f2" "f3" "f4" "f5"])
  (test
    (fields "     f1     f2
     f3 f4    f5 ")
    @["f1" "f2" "f3" "f4" "f5"]))

# (deftest run-cmd
#   (test (run-cmd "echo hello") "hello")
#   (test (run-cmd "ls -d /usr") "/usr")
#   (test-error
#     (run-cmd "/no/such/thing --verbose")
#     "@[\"/no/such/thing\" \"--verbose\"]: No such file or directory"))

(deftest parent
  (test (parent "/") "/")
  (test (parent "/path/to/file") "/path/to"))

(deftest cron-minutes-from-name
  (test (cron-minutes-from-name "tester" 15) "3,18,33,48")
  (test (cron-minutes-from-name "tester" 10) "3,13,23,33,43,53")
  (test (cron-minutes-from-name "tester" 30) "3,33")
  (test (cron-minutes-from-name "test-host-2" 30) "14,44")
  (test (cron-minutes-from-name "test-host-2" 20) "14,34,54"))

(deftest repeated-line-file
  (test
    (repeated-line-file "%d: this is the %s line" [[1 :first] [2 :second] [3 :third]])
    "1: this is the first line\n2: this is the second line\n3: this is the third line\n")

  (test
    (repeated-line-file "this is the %s line" [:first :second :third])
    "this is the first line\nthis is the second line\nthis is the third line\n"))

(deftest indoc
  (test
    (indoc "flat line") "flat line\n")

  (test
    (indoc ```
    line1
    line2
    
    ```)
    "line1\nline2\n\n")

  (test
    (indoc `
      first line indented
    others
    not`)
    "  first line indented\nothers\nnot\n")

  (test
    (indoc `
      gibbus
         and
      chubb`)
    "gibbus\n   and\nchubb\n")

  (test-error
    (indoc 123)
    "indoc: expected a string literal, got 123"))

(deftest template
  (test
    (template-out
      "I, {{ sentiment}}, {{ sentiment }} {{ language }}"
      {:sentiment "like" :language "Janet"})
    "I, like, like Janet")

  (test
    (template-out
      "I {{sentiment    }} {{ language}} too"
      {:sentiment "like" :language "Rust"})
    "I like Rust too")

  (test-error
    (template-out
      "I {{ sentiment }} {{ language }} though"
      {:sentiment "don't much care for" :oops "things like" :language "YAML"})
    "unused vars: expected sentiment, language: got sentiment, language, oops")

  (test-error
    (template-out
      "I also {{ sentiment }} {{ verb }} {{ amount }} of {{ language }}"
      {:sentiment "enjoy" :language "Ruby"})
    "unpopulated fields in template: {{ verb }}, {{ amount }}"))

(deftest qualified-path?
  (test (qualified-path? "/this/is/qualified") true)
  (test (qualified-path? "and/this/is/not") false))

(deftest qualify-from-path-without-dyn
  (test (qualify-from-path "/this/is/qualified") "/this/is/qualified")
  (test-error
    (qualify-from-path "and/this/is/not")
    "cannot qualify path for and/this/is/not: gurp-config-root is not set"))

(deftest qualify-from-path-with-dyn
  (setdyn :gurp-config-root "/test/root")
  (test (qualify-from-path "/this/is/qualified") "/this/is/qualified")
  (test
    (qualify-from-path "some/path")
    "/test/root/files/some/path"))

(deftest tabular-output->struct
  (test (tabular-output->struct
          `
ADDROBJ           TYPE     STATE        ADDR
lo0/v4            static   ok           127.0.0.1/8
build_net0/_a     from-gz  ok           192.168.1.23/24
lo0/v6            static   ok           ::1/128
`)
        {"build_net0/_a" {:addr "192.168.1.23/24"
                          :state "ok"
                          :type "from-gz"}
         "lo0/v4" {:addr "127.0.0.1/8"
                   :state "ok"
                   :type "static"}
         "lo0/v6" {:addr "::1/128"
                   :state "ok"
                   :type "static"}})

  (test (tabular-output->struct
          `LINK        CLASS     MTU    STATE    BRIDGE     OVER
e1000g0     phys      1500   up       --         --
serv_merp0  vnic      1500   up       --         e1000g0
gurp_net0   vnic      1500   up       --         e1000g0
build_net0  vnic      1500   up       --         e1000g0
mariadb_net0 vnic     1500   up       --         e1000g0
pkg_net0    vnic      1500   up       --         e1000g0
records_net0 vnic     1500   up       --         e1000g0`)
        {"build_net0" {:bridge "--"
                       :class "vnic"
                       :mtu 1500
                       :over "e1000g0"
                       :state "up"}
         "e1000g0" {:bridge "--"
                    :class "phys"
                    :mtu 1500
                    :over "--"
                    :state "up"}
         "gurp_net0" {:bridge "--"
                      :class "vnic"
                      :mtu 1500
                      :over "e1000g0"
                      :state "up"}
         "mariadb_net0" {:bridge "--"
                         :class "vnic"
                         :mtu 1500
                         :over "e1000g0"
                         :state "up"}
         "pkg_net0" {:bridge "--"
                     :class "vnic"
                     :mtu 1500
                     :over "e1000g0"
                     :state "up"}
         "records_net0" {:bridge "--"
                         :class "vnic"
                         :mtu 1500
                         :over "e1000g0"
                         :state "up"}
         "serv_merp0" {:bridge "--"
                       :class "vnic"
                       :mtu 1500
                       :over "e1000g0"
                       :state "up"}})

  (test (tabular-output->struct
          `ID NAME             STATUS     PATH                           BRAND    IP
   0 global           running    /                              ipkg     shared
   1 serv-proxy       running    /zones/serv-proxy              lipkg    excl
   2 serv-grafana     running    /zones/serv-grafana            lx       excl
   3 serv-dns         running    /zones/serv-dns                lipkg    excl
   4 serv-gurp        running    /zones/serv-gurp               lipkg    excl
   5 serv-pkg         running    /zones/serv-pkg                lipkg    excl
   6 serv-backup      running    /zones/serv-backup             lipkg    excl
   7 serv-cron        running    /zones/serv-cron               lipkg    excl
   8 serv-metrics     running    /zones/serv-metrics            lipkg    excl
   9 serv-build       running    /zones/serv-build              lipkg    excl
  10 illumos-test     running    /zones/illumos-test            sparse   excl
  11 serv-records     running    /zones/serv-records            pkgsrc   excl
  12 serv-ws          running    /zones/serv-ws                 lipkg    excl
  14 serv-media       running    /zones/serv-media              lipkg    excl
  13 serv-mariadb     running    /zones/serv-mariadb            lipkg    excl
   - lipkg-green      installed  /zones/lipkg-green             lipkg    excl
   - serv-fs          installed  /zones/serv-fs                 lipkg    excl`
          1)
        {"global" {:brand "ipkg"
                   :id 0
                   :ip "shared"
                   :path "/"
                   :status "running"}
         "illumos-test" {:brand "sparse"
                         :id 10
                         :ip "excl"
                         :path "/zones/illumos-test"
                         :status "running"}
         "lipkg-green" {:brand "lipkg"
                        :id "-"
                        :ip "excl"
                        :path "/zones/lipkg-green"
                        :status "installed"}
         "serv-backup" {:brand "lipkg"
                        :id 6
                        :ip "excl"
                        :path "/zones/serv-backup"
                        :status "running"}
         "serv-build" {:brand "lipkg"
                       :id 9
                       :ip "excl"
                       :path "/zones/serv-build"
                       :status "running"}
         "serv-cron" {:brand "lipkg"
                      :id 7
                      :ip "excl"
                      :path "/zones/serv-cron"
                      :status "running"}
         "serv-dns" {:brand "lipkg"
                     :id 3
                     :ip "excl"
                     :path "/zones/serv-dns"
                     :status "running"}
         "serv-fs" {:brand "lipkg"
                    :id "-"
                    :ip "excl"
                    :path "/zones/serv-fs"
                    :status "installed"}
         "serv-grafana" {:brand "lx"
                         :id 2
                         :ip "excl"
                         :path "/zones/serv-grafana"
                         :status "running"}
         "serv-gurp" {:brand "lipkg"
                      :id 4
                      :ip "excl"
                      :path "/zones/serv-gurp"
                      :status "running"}
         "serv-mariadb" {:brand "lipkg"
                         :id 13
                         :ip "excl"
                         :path "/zones/serv-mariadb"
                         :status "running"}
         "serv-media" {:brand "lipkg"
                       :id 14
                       :ip "excl"
                       :path "/zones/serv-media"
                       :status "running"}
         "serv-metrics" {:brand "lipkg"
                         :id 8
                         :ip "excl"
                         :path "/zones/serv-metrics"
                         :status "running"}
         "serv-pkg" {:brand "lipkg"
                     :id 5
                     :ip "excl"
                     :path "/zones/serv-pkg"
                     :status "running"}
         "serv-proxy" {:brand "lipkg"
                       :id 1
                       :ip "excl"
                       :path "/zones/serv-proxy"
                       :status "running"}
         "serv-records" {:brand "pkgsrc"
                         :id 11
                         :ip "excl"
                         :path "/zones/serv-records"
                         :status "running"}
         "serv-ws" {:brand "lipkg"
                    :id 12
                    :ip "excl"
                    :path "/zones/serv-ws"
                    :status "running"}}))

