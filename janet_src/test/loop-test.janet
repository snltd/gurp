(use judge)
(use ../lib/gurp)

(deftest "loop-test-host"
  (role fs
        (zfs/ensure "tank/opt"
                    :properties {:mountpoint "/opt/local"})

        (loop [fs :in ["u01" "u02" "u04"]]
          (add (zfs/ensure (zfscat "tank" fs)
                           :properties {:mountpoint (pathcat "export" fs)
                                        :compression "lz4"
                                        :sharenfs "root=@192.168.1.9/32"
                                        :atime "off"
                                        :exec "off"
                                        :devices "off"
                                        :setuid "off"
                                        :reservation "10G"}))))

  (host "loop-test-host" (fs))

  (test
    (machine-config)
    {:metadata {:name "loop-test-host"}
     :resources {:ensure {:zfs @[{:_id "/fs/zfs/tank_u01"
                                  :action :ensure
                                  :name "tank/u01"
                                  :properties {:atime "off"
                                               :compression "lz4"
                                               :devices "off"
                                               :exec "off"
                                               :mountpoint "/export/u01"
                                               :reservation "10G"
                                               :setuid "off"
                                               :sharenfs "root=@192.168.1.9/32"}
                                  :role "fs"}
                                 {:_id "/fs/zfs/tank_u02"
                                  :action :ensure
                                  :name "tank/u02"
                                  :properties {:atime "off"
                                               :compression "lz4"
                                               :devices "off"
                                               :exec "off"
                                               :mountpoint "/export/u02"
                                               :reservation "10G"
                                               :setuid "off"
                                               :sharenfs "root=@192.168.1.9/32"}
                                  :role "fs"}
                                 {:_id "/fs/zfs/tank_u04"
                                  :action :ensure
                                  :name "tank/u04"
                                  :properties {:atime "off"
                                               :compression "lz4"
                                               :devices "off"
                                               :exec "off"
                                               :mountpoint "/export/u04"
                                               :reservation "10G"
                                               :setuid "off"
                                               :sharenfs "root=@192.168.1.9/32"}
                                  :role "fs"}
                                 {:_id "/fs/zfs/tank_opt"
                                  :action :ensure
                                  :name "tank/opt"
                                  :properties {:mountpoint "/opt/local"}
                                  :role "fs"}]}}}))


(deftest "loop-test-role"
  (test
    ((role test-role
           (pkg/ensure "helix")
           (pkg/ensure "ruby")
           (pkg/ensure "rust")))
    @[{:pkg {:_id "/test-role/pkg/helix"
             :action :ensure
             :name "helix"
             :role "test-role"}}
      {:pkg {:_id "/test-role/pkg/ruby"
             :action :ensure
             :name "ruby"
             :role "test-role"}}
      {:pkg {:_id "/test-role/pkg/rust"
             :action :ensure
             :name "rust"
             :role "test-role"}}])

  (test
    ((role test-role
           (loop [pkg :in ["helix" "ruby" "rust"]] (add (pkg/ensure pkg)))))
    @[{:pkg {:_id "/test-role/pkg/helix"
             :action :ensure
             :name "helix"
             :role "test-role"}}
      {:pkg {:_id "/test-role/pkg/ruby"
             :action :ensure
             :name "ruby"
             :role "test-role"}}
      {:pkg {:_id "/test-role/pkg/rust"
             :action :ensure
             :name "rust"
             :role "test-role"}}
      nil]))
