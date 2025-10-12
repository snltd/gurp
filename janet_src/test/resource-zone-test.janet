(use judge)
(use ../lib/gurp)

(deftest "zone-resources"
  (setdyn :role-dyn "test-role")
  (setdyn :gurp-config-root "/gurpdir")
  (set *collector* (new-collector))

  (zone/ensure "test-zone-thin"
               (zone-network "test_net0"
                             :global-nic "auto"
                             :allowed-address "192.168.1.33/24"
                             :defrouter "192.168.1.1")
               :brand "lipkg")

  (zone/ensure "test-lx-zone"
               (zone-network "test_net0"
                             :global-nic "auto"
                             :allowed-address "192.168.1.33/24"
                             :defrouter "192.168.1.1")
               (zone-attr "kernel-ver" :value "4.4")
               :exec-in ["/bin/exec1" "/bin/exec2"]
               :copy-in {"lx-test/f1" "/etc/file1"
                         "lx-test/f2" "/bin/exec2"}
               :brand "lx")

  (zone/ensure "test-zone-fat"
               :brand "lipkg"
               :autoboot false
               (zone-network "test_net0"
                             :global-nic "auto"
                             :allowed-address "192.168.1.33/24"
                             :defrouter "192.168.1.1")
               (zone-fs "/home" :special "/export/home")
               (zone-fs "/data" :special "/export/data")
               :datasets ["big/zone/fs"]
               :dns {:domain "lan.id264.net"
                     :nameservers ["192.168.1.53"
                                   "192.168.1.1"]}
               :exec-in ["/usr/bin/pkg refresh"])

  (zone/ensure "test-zone-bhyve"
               :brand "bhyve"
               :autoboot false
               (zone-network "test_net0"
                             :allowed-address "192.168.1.33/24"
                             :global-nic "auto")
               (zone-bhyve
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
                         :net @[{:allowed-address "192.168.1.33/24"
                                 :defrouter "192.168.1.1"
                                 :global-nic "auto"
                                 :physical "test_net0"}]
                         :recreate 0
                         :role "test-role"
                         :zonepath "/zones/test-zone-thin"}
                        {:_id "/test-role/zone/test-lx-zone"
                         :attr @[{:name "kernel-ver"
                                  :type "string"
                                  :value "4.4"}]
                         :autoboot true
                         :boot-after-install true
                         :brand "lx"
                         :copy-in {"/gurpdir/files/lx-test/f1" "/etc/file1"
                                   "/gurpdir/files/lx-test/f2" "/bin/exec2"}
                         :exec-in ["/bin/exec1" "/bin/exec2"]
                         :name "test-lx-zone"
                         :net @[{:allowed-address "192.168.1.33/24"
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
                         :fs @[{:dir "/home"
                                :special "/export/home"
                                :type "lofs"}
                               {:dir "/data"
                                :special "/export/data"
                                :type "lofs"}]
                         :name "test-zone-fat"
                         :net @[{:allowed-address "192.168.1.33/24"
                                 :defrouter "192.168.1.1"
                                 :global-nic "auto"
                                 :physical "test_net0"}]
                         :recreate 0
                         :role "test-role"
                         :zonepath "/zones/test-zone-fat"}
                        {:_id "/test-role/zone/test-zone-bhyve"
                         :autoboot false
                         :bhyve {:boot-volume "tank/bhyve/test"
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
                         :net @[{:allowed-address "192.168.1.33/24"
                                 :global-nic "auto"
                                 :physical "test_net0"}]
                         :recreate 0
                         :role "test-role"
                         :zonepath "/zones/test-zone-bhyve"}]}
      :remove @{:zone @[{:_id "/test-role/zone/defunct-zone"
                         :name "defunct-zone"
                         :role "test-role"}]}}))

(deftest "test-zone-rctl-resource"
  (test
    (zone-rctl "zone.max-physical-memory"
               :limit 524288000)
    {:rctl {:action "deny"
            :limit 524288000
            :name "zone.max-physical-memory"
            :priv "privileged"}})

  (test
    (zone-rctl "zone.max-physical-memory"
               :action "allow"
               :limit 12345678)
    {:rctl {:action "allow"
            :limit 12345678
            :name "zone.max-physical-memory"
            :priv "privileged"}})

  (test-error
    (zone-rctl "zone.max-physical-memory")
    "zone-rctl missing required key(s): limit"))

(deftest "test-zone-attr-resource"
  (test
    (zone-attr "turn-it-on" :value false)
    {:attr {:name "turn-it-on"
            :type "boolean"
            :value false}})

  (test
    (zone-attr "spandau-ballet-number-1" :value true :type "astring")
    {:attr {:name "spandau-ballet-number-1"
            :type "astring"
            :value true}})

  (test
    (zone-attr "this-is-a-number" :value 123)
    {:attr {:name "this-is-a-number"
            :type "uint"
            :value 123}})

  (test
    (zone-attr "kernel-ver" :value "4.4")
    {:attr {:name "kernel-ver"
            :type "string"
            :value "4.4"}}))

(deftest "test-zone-attr-error"
  (test-error
    (zone-attr "thing" :type "astring")
    "zone-attr requires a :value"))

(deftest "test-zone-network"
  (test
    (zone-network "test_net0"
                  :allowed-address "1.2.3.4"
                  :defrouter "1.2.3.1")
    {:net {:allowed-address "1.2.3.4"
           :defrouter "1.2.3.1"
           :global-nic "auto"
           :physical "test_net0"}}))
