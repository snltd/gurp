(use judge)
(import ../../../src/doers/zone)

(deftest zone/network
  (test
    (zone/network "test_net0")
    {:net @{:global-nic "auto"
            :physical "test_net0"}})
  (test
    (zone/network "test_net0"
                  :allowed-address "1.2.3.4"
                  :defrouter "1.2.3.1")
    {:net @{:allowed-address "1.2.3.4"
            :defrouter "1.2.3.1"
            :global-nic "auto"
            :physical "test_net0"}})

  (test-error
    (zone/network "test_net0" :oops "wat?")
    "unexpected property :oops. Valid properties are :physical, :global-nic, :defrouter, :allowed-address, :label"))
