(use judge)
(import ../../../src/doers/zone)

(deftest zone/network
  (test
    (zone/network "test_net0")
    {:net @{:global-nic "auto"
            :physical "test_net0"}})
  (test
    (zone/network "test_net1"
                  :allowed-address "1.2.3.4"
                  :defrouter "1.2.3.1")
    {:net @{:allowed-address "1.2.3.4"
            :defrouter "1.2.3.1"
            :global-nic "auto"
            :physical "test_net1"}})

  (test-error
    (zone/network "test_net_2"
                  :global-nic 0)
    "In zone/network NO-NAME: global-nic is of type :number. Allowed types :string")

  (test-error
    (zone/network "test_net3" :oops "wat?")
    "In zone/network NO-NAME: unexpected property :oops. Valid properties are :physical, :global-nic, :defrouter, :allowed-address, :label"))
