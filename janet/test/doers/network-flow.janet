(use judge)
(use ./_helpers)
(use ../../src/collector)
(import ../../src/doers/network-flow)

(deftest network-flow
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (import-tests "network-flow" (curenv))
  (network-flow/ensure "cap_all"
                       :link "vnic0"
                       :maxbw "50M")

  (network-flow/ensure "flow-www-test"
                       :link "vnic0"
                       :protocol "tcp"
                       :local-port 80
                       :maxbw "10M"
                       :priority "high")


  (network-flow/ensure "flow-nic-test"
                       :link "vnic0"
                       :maxbw "1M")

  (test *collector*
    @{:ensure @{:network-flow @[{:_id "/test-role/network-flow/tls_shaper"
                                 :link "vnic1"
                                 :maxbw "10M"
                                 :name "tls_shaper"
                                 :protocol "tcp"
                                 :remote-ip "203.0.113.4"
                                 :remote-port 443
                                 :role "test-role"}
                                {:_id "/test-role/network-flow/flow-ssh-test"
                                 :link "vnic0"
                                 :local-port 22
                                 :maxbw "1M"
                                 :name "flow-ssh-test"
                                 :protocol "tcp"
                                 :role "test-role"}
                                {:_id "/test-role/network-flow/cap_all"
                                 :link "vnic0"
                                 :maxbw "50M"
                                 :name "cap_all"
                                 :role "test-role"}
                                {:_id "/test-role/network-flow/flow-www-test"
                                 :link "vnic0"
                                 :local-port 80
                                 :maxbw "10M"
                                 :name "flow-www-test"
                                 :priority "high"
                                 :protocol "tcp"
                                 :role "test-role"}
                                {:_id "/test-role/network-flow/flow-nic-test"
                                 :link "vnic0"
                                 :maxbw "1M"
                                 :name "flow-nic-test"
                                 :role "test-role"}]}
      :remove @{:network-flow @[{:_id "/test-role/network-flow/unwanted"
                                 :name "unwanted"
                                 :role "test-role"}]}}))

(deftest network-flow-errors
  (test-error
    (network-flow/ensure "extraneous-property"
                         :this-should-break-it true
                         :link "vnic0"
                         :protocol "tcp"
                         :local-port 80
                         :maxbw "10M"
                         :priority "high")
    "unexpected property :this-should-break-it. Valid properties are :link, :dsfield, :remote-port, :remote-ip, :priority, :protocol, :label, :local-port, :maxbw, :local-ip")

  (test-error
    (network-flow/ensure "missing-link"
                         :protocol "tcp"
                         :local-port 80
                         :maxbw "10M"
                         :priority "high")
    "did not find mandatory property :link. Mandatory properties are :link"))
