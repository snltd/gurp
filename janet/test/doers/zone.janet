(use judge)
(use ./lib/test-lib)
(use ../../src/collector)
(use ../../src/dsl)
(import ../../src/doers/zone)

(deftest zone
  (setdyn :role-dyn "test-role")
  (setdyn :gurp-config-root "/gurpdir")
  (set *collector* (new-collector))

  (import-tests "zone")

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
    @{:ensure @{:zone @[{:_id "/test-role/zone/bhyve-zone"
                         :autoboot false
                         :bhyve @{:boot-rom "BHYVE_RELEASE"
                                  :boot-volume "tank/bhyve/test"
                                  :ram "4G"
                                  :vcpus 4
                                  :wait-for-boot true}
                         :boot-after-install true
                         :brand "bhyve"
                         :cloudinit @{:from @{"packages" "/gurpdir/files/cloudwatch/packages"
                                              "users" "/gurpdir/files/cloudwatch/users"}
                                      :from-struct @{"meta-data" {:instance-id "example-zone"
                                                                  :local-hostname "example-zone"}
                                                     "network" {:network {:fancy "struct"}}}}
                         :image "/var/tmp/noble-server-cloudimg-amd64.img.raw"
                         :name "bhyve-zone"
                         :network @[@{:allowed-address "10.10.0.2/24"
                                      :global-nic "auto"
                                      :name "bhyve0"}]
                         :role "test-role"}
                        {:_id "/test-role/zone/native-zone"
                         :autoboot true
                         :boot-after-install true
                         :bootstrap @{:copy-self true
                                      :gurp-binary "/var/tmp/gurp"
                                      :hostname "native-zone"
                                      :server "gurp.localnet"}
                         :brand "lipkg"
                         :clone-from "gold-zone"
                         :fs @[@{:name "/home"
                                 :special "/export/home"
                                 :type "lofs"}]
                         :name "native-zone"
                         :network @[@{:allowed-address "192.168.1.101/24"
                                      :defrouter "192.168.1.1"
                                      :global-nic "auto"
                                      :name "test_net0"}]
                         :role "test-role"}
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
                         :image "alpine"
                         :name "lx-zone"
                         :network @[@{:allowed-address "192.168.1.103/24"
                                      :defrouter "192.168.1.1"
                                      :global-nic "auto"
                                      :name "znet0"}]
                         :role "test-role"}
                        {:_id "/test-role/zone/emu-example"
                         :attr @[@{:name "type"
                                   :type "string"
                                   :value "generic"}
                                 @{:name "diskif"
                                   :type "string"
                                   :value "virtio-blk-device"}
                                 @{:name "netif"
                                   :type "string"
                                   :value "virtio-net-device"}]
                         :autoboot true
                         :boot-after-install true
                         :brand "emu"
                         :emu @{:arch "aarch64"
                                :bios "https://downloads.omnios.org/media/braich/u-boot.bin"
                                :boot-volume "tank/emu-example"
                                :cpu "cortex-a53"
                                :image-format "raw"
                                :qemu-args @["\"-machine virt\""
                                             "\"-accel tcg,thread=multi\""]
                                :ram "2G"
                                :vcpus 4
                                :wait-for-boot true}
                         :image "/var/tmp/braich-151059.raw"
                         :name "emu-example"
                         :network @[@{:allowed-address "192.168.1.102/24"
                                      :global-nic "auto"
                                      :name "emu0"}]
                         :rctl @[@{:action "none"
                                   :limit 1
                                   :name "zone.cpu-shares"
                                   :priv "privileged"}]
                         :recreate 1
                         :role "test-role"}
                        {:_id "/test-role/zone/test-zone-bootstrap-file"
                         :autoboot true
                         :boot-after-install true
                         :bootstrap @{:copy-self true
                                      :file "/var/tmp/bootstrap.janet"
                                      :gurp-binary "/var/tmp/gurp"}
                         :brand "lipkg"
                         :name "test-zone-bootstrap-file"
                         :network @[@{:allowed-address "192.168.1.33/24"
                                      :defrouter "192.168.1.1"
                                      :global-nic "auto"
                                      :name "test_net0"}]
                         :role "test-role"}
                        {:_id "/test-role/zone/test-zone-fat"
                         :autoboot false
                         :boot-after-install true
                         :brand "lipkg"
                         :datasets ["big/zone/fs"]
                         :dns {:domain "lan.id264.net"
                               :nameservers ["192.168.1.53" "192.168.1.1"]}
                         :exec-in ["/usr/bin/pkg refresh"]
                         :fs @[@{:name "/home"
                                 :special "/export/home"
                                 :type "lofs"}
                               @{:name "/data"
                                 :special "/export/data"
                                 :type "lofs"}]
                         :name "test-zone-fat"
                         :network @[@{:allowed-address "192.168.1.33/24"
                                      :defrouter "192.168.1.1"
                                      :global-nic "auto"
                                      :name "test_net0"}]
                         :role "test-role"}]}
      :remove @{:zone @[{:_id "/test-role/zone/unwanted-zone"
                         :name "unwanted-zone"
                         :role "test-role"}]}}))

 (deftest zone-error
   (test-error
     (zone/ensure "wat-brand")
     "In zone/ensure wat-brand: did not find mandatory property :brand. Mandatory properties are :autoboot, :boot-after-install, :brand")

   (test-error
     (zone/ensure "bad-key"
                  :brand "sparse"
                  :oops "wat")
     "In zone/ensure bad-key: unexpected property :oops. Valid properties are :attr, :autoboot, :bhyve, :boot-after-install, :bootstrap, :brand, :capped-memory, :checksum, :clone-from, :cloudinit, :copy-in, :datasets, :dns, :emu, :exec-in, :final-state, :fs, :hostid, :image, :ip-type, :label, :limitpriv, :network, :pool, :rctl, :recreate, :zonepath"))
