(use judge)
(import ../../../src/doers/zone)

(deftest zone/network
  (test
    (zone/network "test_net0")
    {:network @{:global-nic "auto" :name "test_net0"}})
  (test
    (zone/network "test_net1"
                  :allowed-address "1.2.3.4"
                  :defrouter "1.2.3.1")
    {:network @{:allowed-address "1.2.3.4"
                :defrouter "1.2.3.1"
                :global-nic "auto"
                :name "test_net1"}})

  (test-error
    (zone/network "test_net_2"
                  :global-nic 0)
    "In zone/network test_net_2: global-nic is of type :number. Allowed types :string")

  (test-error
    (zone/network "test_net3" :oops "wat?")
    "In zone/network test_net3: unexpected property :oops. Valid properties are :allowed-address, :defrouter, :global-nic, :label, :name"))
