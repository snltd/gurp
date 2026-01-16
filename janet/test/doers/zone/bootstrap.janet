(use judge)
(import ../../../src/doers/zone)

(deftest zone/bootstrap
  (test
    (zone/bootstrap :server "gurp.localnet"
                    :hostname "test-client")
    {:bootstrap @{:hostname "test-client"
                  :server "gurp.localnet"}})

  (test
    (zone/bootstrap :file "/var/tmp/boot.janet")
    {:bootstrap @{:file "/var/tmp/boot.janet"}})

  (test-error
    (zone/bootstrap :oops "wat?")
    "unexpected property :oops. Valid properties are :label, :file, :server, :hostname"))
