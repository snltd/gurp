(use judge)
(use ../lib/gurp)

(deftest "test-zone-network"
  (test
    (zone-network "test_net0" :allowed-address: "1.2.3.4" :defrouter "1.2.3.1" )
    {:allowed-address: "1.2.3.4"
     :defrouter "1.2.3.1"
     :global-nic "auto"
     :physical "test_net0"}))

(deftest "test zone padding"
  (setdyn :role-dyn "test-role")
  (test
    (zone/ensure "test-zone"
                 :brand "lipkg")
    {:zone {:_id "/test-role/zone/test-zone"
            :action :ensure
            :autoboot true
            :boot-after-install true
            :brand "lipkg"
            :name "test-zone"
            :recreate 0
            :role "test-role"
            :zonepath "/zones/test-zone"}}))

(deftest "test zone functions"
  (setdyn :role-dyn "test-role")
  (test
    (zone/ensure "test-zone"
                 :brand "lipkg"
                 :autoboot false
                 (zone-network "fs_net0"
                               :global-nic "auto"
                               :allowed-address "192.168.1.33/24"
                               :defrouter "192.168.1.1")
                 (zone-fs "/home" :special "/export/home")
                 (zone-fs "/data" :special "/export/data")
                 :datasets ["big/zone/fs"]
                 :dns {:domain "lan.id264.net"
                       :nameservers ["192.168.1.53"
                                     "192.168.1.1"]}
                 :run-cmd ["/usr/bin/pkg refresh"])
    {:zone {:_id "/test-role/zone/test-zone"
            :action :ensure
            :autoboot false
            :boot-after-install true
            :brand "lipkg"
            :datasets ["big/zone/fs"]
            :dns {:domain "lan.id264.net"
                  :nameservers ["192.168.1.53" "192.168.1.1"]}
            :fs @[{:dir "/home"
                   :special "/export/home"
                   :type "lofs"}
                  {:dir "/data"
                   :special "/export/data"
                   :type "lofs"}]
            :name "test-zone"
            :networks @[{:allowed-address "192.168.1.33/24"
                         :defrouter "192.168.1.1"
                         :global-nic "auto"
                         :physical "fs_net0"}]
            :recreate 0
            :role "test-role"
            :run-cmd ["/usr/bin/pkg refresh"]
            :zonepath "/zones/test-zone"}})

  (test
    (zone/remove "defunct-zone")
    {:zone {:_id "/test-role/zone/defunct-zone"
            :action :remove
            :name "defunct-zone"
            :role "test-role"}}))
