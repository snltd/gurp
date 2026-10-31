(use judge)
(import ../../../src/doers/zone)

(deftest zone/bootstrap
  (test
    (zone/bootstrap :server "gurp.localnet"
                    :hostname "test-client")
    [:bootstrap
     @{:copy-self true
       :from-helper true
       :gurp-binary "/var/tmp/gurp"
       :hostname "test-client"
       :server "gurp.localnet"
       :unique false}])

  (test
    (zone/bootstrap :file "/var/tmp/boot.janet")
    [:bootstrap
     @{:copy-self true
       :file "/var/tmp/boot.janet"
       :from-helper true
       :gurp-binary "/var/tmp/gurp"
       :unique false}])

  (test-error
    (zone/bootstrap :oops "wat?")
    "In zone/bootstrap NO-NAME: need exactly one of :file, :server")

  (test-error
    (zone/bootstrap)
    "In zone/bootstrap NO-NAME: need exactly one of :file, :server")
    
  (test-error
    (zone/bootstrap :server "gurp.localnet"
                    :file "/var/tmp/config.janet")
    "In zone/bootstrap NO-NAME: need exactly one of :file, :server"))
