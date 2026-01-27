(zone/ensure "native-zone"
             :brand "lipkg"
             :clone-from "gold-zone"
             (zone/fs "/home"
                      :special "/export/home")
             (zone/network "test_net0"
                           :global-nic "auto"
                           :allowed-address "192.168.1.101/24"
                           :defrouter "192.168.1.1")
             (zone/bootstrap
               :server "gurp.localnet"
               :hostname "native-zone"))
