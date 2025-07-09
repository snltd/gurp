(use judge)
(use ../lib/gurp)

(deftest "test zone functions"
  (setdyn :role-dyn "test-role")
  (test
    (zone/ensure "test-zone"
    :brand "lipkg"
    :zonepath "/zones/serv-fs"
    :autoboot false
    :networks [{:physical "fs_net0"
               :global-nic "auto"
               :allowed-address "192.168.1.33/24"
               :defrouter "192.168.1.1"}]
    :fs [{:dir "/home"
          :special "/export/home"
          :type "lofs"}]
    :datasets ["big/zone/fs"]
    :dns {:domain "lan.id264.net"
          :nameservers ["192.168.1.53"
                        "192.168.1.1"]}
    :run-cmd ["/usr/bin/pkg refresh"])
    {:zone {:_id "/test-role/zone/test-zone"
            :action :ensure
            :autoboot false
            :brand "lipkg"
            :datasets ["big/zone/fs"]
            :dns {:domain "lan.id264.net"
                  :nameservers ["192.168.1.53" "192.168.1.1"]}
            :fs [{:dir "/home"
                  :special "/export/home"
                  :type "lofs"}]
            :name "test-zone"
            :networks [{:allowed-address "192.168.1.33/24"
                        :defrouter "192.168.1.1"
                        :global-nic "auto"
                        :physical "fs_net0"}]
            :role "test-role"
            :run-cmd ["/usr/bin/pkg refresh"]
            :zonepath "/zones/serv-fs"}})

  (test
    (zone/remove "defunct-zone")
    {:zone {:_id "/test-role/zone/defunct-zone"
            :action :remove
            :name "defunct-zone"
            :role "test-role"}}))
