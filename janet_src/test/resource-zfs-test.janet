(use judge)
(use ../lib/gurp)

(deftest "zfs-resources"
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (zfs/ensure (zfscat "tank" "export" "test")
              :label "test-zfs"
              :properties {:compression "gzip9"
                           :devices "off"})

  (zfs/ensure (zfscat "tank" "export" "test-vol")
              :size "10G"
              :label "test-zfs-vol"
              :properties {:devices "off"})

  (zfs/remove "old/filesystem")

  (test *collector*
        @{:ensure @{:zfs @[{:_id "/test-role/zfs/test-zfs"
                            :label "test-zfs"
                            :name "tank/export/test"
                            :properties {:compression "gzip9" :devices "off"}
                            :role "test-role"}
                           {:_id "/test-role/zfs/test-zfs-vol"
                            :label "test-zfs-vol"
                            :name "tank/export/test-vol"
                            :properties {:devices "off"}
                            :role "test-role"
                            :size "10G"}]}
          :remove @{:zfs @[{:_id "/test-role/zfs/old_filesystem"
                            :name "old/filesystem"
                            :role "test-role"}]}}))

(deftest "zfs-errors"
  (test-error
    (zfs/ensure "pool/fs"
      :properties {:mountpoint "none"}
      :volume-size "100M")
    "Failed to validate user input for zfs 'pool/fs': zfs 'pool/fs' has unrecognised key(s): volume-size"))
