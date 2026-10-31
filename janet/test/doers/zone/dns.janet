(use judge)
(use ../test-lib)
(import ../../../src/doers/zone)

(deftest zone/dns
  (test
    (import-test "zone/dns-01.janet")
    {:dns @{:domain "example.com"
            :nameservers ["192.168.1.53" "1.1.1.1" "8.8.8.8"]
            :options ["attempts:5" "rotate"]}})

  (test-error
    (zone/dns :domain "example.com")
    "In zone/dns NO-NAME: did not find mandatory property :nameservers. Mandatory properties are :nameservers, :domain")

  (test-error
    (zone/dns :domain "example.com"
              :nameservers "1.1.1.1")
    "In zone/dns NO-NAME: nameservers is of type :string. Allowed types :tuple")

  (test-error
    (zone/dns :domain "example.com"
              :nameservers ["1.1.1.1"]
              :attempts 5)
    "In zone/dns NO-NAME: unexpected property :attempts. Valid properties are :domain, :label, :nameservers, :options, :search-list"))
