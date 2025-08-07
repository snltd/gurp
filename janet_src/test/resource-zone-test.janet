(use judge)
(use ../lib/gurp)

(test-macro
  (expand-zone-fn :attr)
  (do
    (def is-key (group-by (short-fn (and (struct? $) (deep= @[:attr] (keys $)))) modified-specs))
    (if-let [key-list (is-key true)]
      (set modified-specs (tuple (splice (is-key false)) :attr (mapcat values key-list))))))

(deftest "zone-resources"
  (setdyn :role-dyn "test-role")
  (setdyn :gurp-config-root "/gurpdir")
  (set *collector* (new-collector))

  (zone/ensure "test-zone-thin"
               :brand "lipkg")

  (zone/ensure "test-lx-zone"
               (zone-attr "kernel-ver" :value "4.4")
               :exec-in ["/bin/exec1" "/bin/exec2"]
               :copy-in {"lx-test/f1" "/etc/file1"
                         "lx-test/f2" "/bin/exec2"}
               :brand "lx")

  (zone/ensure "test-zone-fat"
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

  (zone/remove "defunct-zone")

  (test *collector*
        @{:ensure @{:zone @[{:_id "/test-role/zone/test-zone-thin"
                             :autoboot true
                             :boot-after-install true
                             :brand "lipkg"
                             :name "test-zone-thin"
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
                                     :physical "fs_net0"}]
                             :recreate 0
                             :role "test-role"
                             :run-cmd ["/usr/bin/pkg refresh"]
                             :zonepath "/zones/test-zone-fat"}]}

          :remove @{:zone @[{:_id "/test-role/zone/defunct-zone"
                             :name "defunct-zone"
                             :role "test-role"}]}}))

(deftest "test-zone-rctl-resource"
  (test
    (zone-rctl "zone.max-physical-memory"
      :value 524288000)
    {:rctl {:action "deny"
            :name "zone.max-physical-memory"
            :priv "privileged"
            :value 524288000}}) )

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
    (zone-network "test_net0" :allowed-address: "1.2.3.4" :defrouter "1.2.3.1")
    {:net {:allowed-address: "1.2.3.4"
           :defrouter "1.2.3.1"
           :global-nic "auto"
           :physical "test_net0"}}))
