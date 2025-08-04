(use judge)
(use ../lib/gurp)

(deftest "zfs-resources"
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (zfs/ensure (zfscat "tank" "export" "test")
              :properties {:label "test-zfs"
                           :compression "gzip9"
                           :devices "off"})

  (zfs/remove "old/filesystem")

  (test *collector*
        @{:ensure @{:zfs @[{:_id "/test-role/zfs/tank_export_test"
                            :name "tank/export/test"
                            :properties {:compression "gzip9"
                                         :devices "off"
                                         :label "test-zfs"}
                            :role "test-role"}]}
          :remove @{:zfs @[{:_id "/test-role/zfs/old_filesystem"
                            :name "old/filesystem"
                            :role "test-role"}]}}))

(deftest "zfs-errors"
  (test-error
    (zfs/ensure "pool/fs" :properties {:mountpoint "none"} :size "100M")
    "Failed to validate user input for zfs 'pool/fs' : zfs 'pool/fs' has unrecognised key(s): size"))
