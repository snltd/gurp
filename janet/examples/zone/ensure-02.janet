(zone/ensure "test-zone-bootstrap-net"
             (zone/network "test_net0"
                           :global-nic "auto"
                           :allowed-address "192.168.1.33/24"
                           :defrouter "192.168.1.1")
             (zone/bootstrap
               :server "gurp.localnet"
               :hostname "test-zone-bootstrap")
             :brand "lipkg")
