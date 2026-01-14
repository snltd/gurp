(use judge)
(use ../../src/collector)
(import ../../src/doers/zone)

(deftest zone
  (setdyn :role-dyn "test-role")
  (setdyn :gurp-config-root "/gurpdir")
  (set *collector* (new-collector))

  (zone/ensure "test-zone-thin"
               (zone/network "test_net0"
                             :global-nic "auto"
                             :allowed-address "192.168.1.33/24"
                             :defrouter "192.168.1.1")
               :brand "lipkg")

  (zone/ensure "test-zone-bootstrap-net"
               (zone/network "test_net0"
                             :global-nic "auto"
                             :allowed-address "192.168.1.33/24"
                             :defrouter "192.168.1.1")
               (zone/bootstrap
                 :server "gurp.localnet"
                 :hostname "test-zone-bootstrap")
               :brand "lipkg")

  (zone/ensure "test-zone-bootstrap-file"
               (zone/network "test_net0"
                             :global-nic "auto"
                             :allowed-address "192.168.1.33/24"
                             :defrouter "192.168.1.1")
               (zone/bootstrap :file "/var/tmp/bootstrap.janet")
               :brand "lipkg")

  (zone/ensure "test-lx-zone"
               (zone/network "test_net0"
                             :global-nic "auto"
                             :allowed-address "192.168.1.33/24"
                             :defrouter "192.168.1.1")
               (zone/attr "kernel-ver" :value "4.4")
               :exec-in ["/bin/exec1" "/bin/exec2"]
               :copy-in {"lx-test/f1" "/etc/file1"
                         "lx-test/f2" "/bin/exec2"}
               :brand "lx")

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

  (zone/ensure "test-zone-bhyve"
               :brand "bhyve"
               :autoboot false
               (zone/network "test_net0"
                             :allowed-address "192.168.1.33/24"
                             :global-nic "auto")
               (zone/bhyve
                 :ram "3G"
                 :vcpus 4
                 :image-path "/var/tmp/noble-server-cloudimg-amd64.img.raw"
                 :boot-volume "tank/bhyve/test"
                 :cloudinit-struct {:network {:version 2}})
               :dns {:domain "lan.id264.net"
                     :nameservers ["192.168.1.53"
                                   "192.168.1.1"]})

  (zone/remove "defunct-zone")

  (test *collector*
        @{:ensure @{:zone @[{:_id "/test-role/zone/test-zone-thin"
                             :autoboot true
                             :boot-after-install true
                             :brand "lipkg"
                             :name "test-zone-thin"
                             :net @[@{:allowed-address "192.168.1.33/24"
                                      :defrouter "192.168.1.1"
                                      :global-nic "auto"
                                      :physical "test_net0"}]
                             :recreate 0
                             :role "test-role"
                             :zonepath "/zones/test-zone-thin"}
                            {:_id "/test-role/zone/test-zone-bootstrap-net"
                             :autoboot true
                             :boot-after-install true
                             :bootstrap @{:hostname "test-zone-bootstrap"
                                          :server "gurp.localnet"}
                             :brand "lipkg"
                             :name "test-zone-bootstrap-net"
                             :net @[@{:allowed-address "192.168.1.33/24"
                                      :defrouter "192.168.1.1"
                                      :global-nic "auto"
                                      :physical "test_net0"}]
                             :recreate 0
                             :role "test-role"
                             :zonepath "/zones/test-zone-bootstrap-net"}
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
                            {:_id "/test-role/zone/test-lx-zone"
                             :attr @[@{:name "kernel-ver"
                                       :type "string"
                                       :value "4.4"}]
                             :autoboot true
                             :boot-after-install true
                             :brand "lx"
                             :copy-in @{"/gurpdir/files/lx-test/f1" "/etc/file1"
                                        "/gurpdir/files/lx-test/f2" "/bin/exec2"}
                             :exec-in ["/bin/exec1" "/bin/exec2"]
                             :name "test-lx-zone"
                             :net @[@{:allowed-address "192.168.1.33/24"
                                      :defrouter "192.168.1.1"
                                      :global-nic "auto"
                                      :physical "test_net0"}]
                             :recreate 0
                             :role "test-role"
                             :zonepath "/zones/test-lx-zone"}
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
                             :zonepath "/zones/test-zone-fat"}
                            {:_id "/test-role/zone/test-zone-bhyve"
                             :autoboot false
                             :bhyve @{:boot-volume "tank/bhyve/test"
                                      :cloudinit-struct {:network {:version 2}}
                                      :image-path "/var/tmp/noble-server-cloudimg-amd64.img.raw"
                                      :ram "3G"
                                      :vcpus 4
                                      :wait-for-boot true}
                             :boot-after-install true
                             :brand "bhyve"
                             :dns {:domain "lan.id264.net"
                                   :nameservers ["192.168.1.53" "192.168.1.1"]}
                             :name "test-zone-bhyve"
                             :net @[@{:allowed-address "192.168.1.33/24"
                                      :global-nic "auto"
                                      :physical "test_net0"}]
                             :recreate 0
                             :role "test-role"
                             :zonepath "/zones/test-zone-bhyve"}]}
          :remove @{:zone @[{:_id "/test-role/zone/defunct-zone"
                             :name "defunct-zone"
                             :role "test-role"}]}}))
