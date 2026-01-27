(use judge)
(use ./_helpers)
(use ../../src/collector)
(import ../../src/doers/zone)

(deftest zone
  (setdyn :role-dyn "test-role")
  (setdyn :gurp-config-root "/gurpdir")
  (set *collector* (new-collector))

  (import-tests "zone" (curenv))

  (zone/ensure "test-zone-bootstrap-file"
               (zone/network "test_net0"
                             :global-nic "auto"
                             :allowed-address "192.168.1.33/24"
                             :defrouter "192.168.1.1")
               (zone/bootstrap :file "/var/tmp/bootstrap.janet")
               :brand "lipkg")

  (zone/ensure "test-zone-fat"
               :brand "lipkg"
               :autoboot false
               (zone/network "test_net0"
                             :global-nic "auto"
                             :allowed-address "192.168.1.33/24"
                             :defrouter "192.168.1.1")
               (zone/fs "/home" :special "/export/home")
               (zone/fs "/data" :special "/export/data")
               :datasets ["big/zone/fs"]
               :dns {:domain "lan.id264.net"
                     :nameservers ["192.168.1.53"
                                   "192.168.1.1"]}
               :exec-in ["/usr/bin/pkg refresh"])

  (test *collector*
    @{:ensure @{:zone @[{:_id "/test-role/zone/native-zone"
                         :autoboot true
                         :boot-after-install true
                         :bootstrap @{:hostname "native-zone"
                                      :server "gurp.localnet"}
                         :brand "lipkg"
                         :clone-from "gold-zone"
                         :fs @[@{:dir "/home"
                                 :special "/export/home"
                                 :type "lofs"}]
                         :name "native-zone"
                         :net @[@{:allowed-address "192.168.1.101/24"
                                  :defrouter "192.168.1.1"
                                  :global-nic "auto"
                                  :physical "test_net0"}]
                         :recreate 0
                         :role "test-role"
                         :zonepath "/zones/native-zone"}
                        {:_id "/test-role/zone/bhyve-zone"
                         :autoboot false
                         :bhyve @{:boot-volume "tank/bhyve/test"
                                  :cloudinit-struct {:network {:version 2}}
                                  :image-path "/var/tmp/noble-server-cloudimg-amd64.img.raw"
                                  :ram "4G"
                                  :vcpus 4
                                  :wait-for-boot true}
                         :boot-after-install true
                         :brand "bhyve"
                         :dns {:domain "lan.id264.net"
                               :nameservers ["192.168.1.53" "192.168.1.1"]}
                         :name "bhyve-zone"
                         :net @[@{:allowed-address "192.168.1.102/24"
                                  :global-nic "auto"
                                  :physical "bhyve0"}]
                         :recreate 0
                         :role "test-role"
                         :zonepath "/zones/bhyve-zone"}
                        {:_id "/test-role/zone/lx-zone"
                         :attr @[@{:name "kernel-ver"
                                   :type "string"
                                   :value "4.4"}]
                         :autoboot true
                         :boot-after-install true
                         :brand "lx"
                         :copy-in @{"/gurpdir/files/lx-test/f1" "/etc/file1"
                                    "/gurpdir/files/lx-test/f2" "/bin/exec2"}
                         :exec-in ["/bin/exec1" "/bin/exec2"]
                         :final-state "reboot"
                         :lx-image "alpine"
                         :name "lx-zone"
                         :net @[@{:allowed-address "192.168.1.103/24"
                                  :defrouter "192.168.1.1"
                                  :global-nic "auto"
                                  :physical "znet0"}]
                         :recreate 0
                         :role "test-role"
                         :zonepath "/zones/lx-zone"}
                        {:_id "/test-role/zone/test-zone-bootstrap-file"
                         :autoboot true
                         :boot-after-install true
                         :bootstrap @{:file "/var/tmp/bootstrap.janet"}
                         :brand "lipkg"
                         :name "test-zone-bootstrap-file"
                         :net @[@{:allowed-address "192.168.1.33/24"
                                  :defrouter "192.168.1.1"
                                  :global-nic "auto"
                                  :physical "test_net0"}]
                         :recreate 0
                         :role "test-role"
                         :zonepath "/zones/test-zone-bootstrap-file"}
                        {:_id "/test-role/zone/test-zone-fat"
                         :autoboot false
                         :boot-after-install true
                         :brand "lipkg"
                         :datasets ["big/zone/fs"]
                         :dns {:domain "lan.id264.net"
                               :nameservers ["192.168.1.53" "192.168.1.1"]}
                         :exec-in ["/usr/bin/pkg refresh"]
                         :fs @[@{:dir "/home"
                                 :special "/export/home"
                                 :type "lofs"}
                               @{:dir "/data"
                                 :special "/export/data"
                                 :type "lofs"}]
                         :name "test-zone-fat"
                         :net @[@{:allowed-address "192.168.1.33/24"
                                  :defrouter "192.168.1.1"
                                  :global-nic "auto"
                                  :physical "test_net0"}]
                         :recreate 0
                         :role "test-role"
                         :zonepath "/zones/test-zone-fat"}]}
      :remove @{:zone @[{:_id "/test-role/zone/unwanted-zone"
                         :name "unwanted-zone"
                         :role "test-role"}]}}))
