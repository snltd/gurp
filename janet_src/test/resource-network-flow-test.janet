(use judge)
(use ../lib/gurp)

(deftest "network-flow-test"
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (network-flow/ensure "cap_all"
                       :link "vnic0"
                       :maxbw "50M")

  (network-flow/ensure "tls_shaper"
                       :link "vnic1"
                       :protocol "tcp"
                       :remote-ip "203.0.113.4"
                       :remote-port 443
                       :maxbw "10M")
  # (network-flow/ensure "flow-www-test"
  #                      :nic "vnic0"
  #                      :attributes {:transport "tcp"
  #                                   :local_port 80}
  #                      :properties {:maxbw "10M"
  #                                   :priority "high"})

  # (network-flow/ensure "flow-ssh-test"
  #                      :nic "vnic0"
  #                      :attributes {:transport "tcp"
  #                                   :local_port 22}
  #                      :properties {:maxbw "1M"})


  # (network-flow/ensure "flow-nic-test"
  #                      :nic "vnic0"
  #                      :properties {:maxbw "1M"})

  (test *collector*
    @{:ensure @{:network-flow @[{:_id "/test-role/network-flow/cap_all"
                                 :link "vnic0"
                                 :maxbw "50M"
                                 :name "cap_all"
                                 :role "test-role"}
                                {:_id "/test-role/network-flow/tls_shaper"
                                 :link "vnic1"
                                 :maxbw "10M"
                                 :name "tls_shaper"
                                 :protocol "tcp"
                                 :remote-ip "203.0.113.4"
                                 :remote-port 443
                                 :role "test-role"}]}
      :remove @{}}))

# (deftest "network-flow-errors"
#   (test-error
#     (network-flow/ensure "extraneous-property"
#                          :this-should-break-it true
#                          :nic "vnic0"
#                          :attributes {:transport "tcp"
#                                       :local_port 80}
#                          :properties {:maxbw "10M"
#                                       :priority "high"})
#     "Failed to validate user input for network-flow 'extraneous-property': network-flow 'extraneous-property' has unrecognised key(s): this-should-break-it")

#   (test-error
#     (network-flow/ensure "missing-nic"
#                          :attributes {:transport "tcp"
#                                       :local_port 80}
#                          :properties {:maxbw "10M"
#                                       :priority "high"})
#     "Failed to validate user input for network-flow 'missing-nic': network-flow missing required key(s): nic")
#   (test-error
#     (network-flow/ensure "missing-properties"
#                          :nic "vnic0"
#                          :attributes {:transport "tcp"
#                                       :local_port 80})
#     "Failed to validate user input for network-flow 'missing-properties': network-flow missing required key(s): properties"))

