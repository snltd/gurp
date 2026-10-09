(use judge)
(import ../../../src/doers/zone)

(deftest zone/bootstrap
  (test
    (zone/bootstrap :server "gurp.localnet"
                    :hostname "test-client")
    {:bootstrap @{:copy-self true
                  :debug false
                  :gurp-binary "/var/tmp/gurp"
                  :hostname "test-client"
                  :server "gurp.localnet"}})

  (test
    (zone/bootstrap :file "/var/tmp/boot.janet")
    {:bootstrap @{:copy-self true
                  :debug false
                  :file "/var/tmp/boot.janet"
                  :gurp-binary "/var/tmp/gurp"}})

  (test-error
    (zone/bootstrap :oops "wat?")
    "In zone/bootstrap NO-NAME: unexpected property :oops. Valid properties are :label, :hostname, :copy-self, :debug, :gurp-binary, :server, :file"))
