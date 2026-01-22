(zone/ensure "test-zone-thin"
             (zone/network "test_net0"
                           :global-nic "auto"
                           :allowed-address "192.168.1.33/24"
                           :defrouter "192.168.1.1")
             :brand "lipkg")
