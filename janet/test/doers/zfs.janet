(use judge)
(use ./test-lib)
(use ../../src/collector)
(use ../../src/dsl)
(import ../../src/doers/zfs)

(deftest zfs
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (import-tests "zfs" (curenv))

  (test *collector*
        @{:ensure @{:zfs @[{:_id "/test-role/zfs/zfs-example-1"
                            :label "zfs-example-1"
                            :name "tank/example/filesystem"
                            :properties {:compression "gzip9"
                                         :dedup true
                                         :devices false
                                         :mountpoint "/example/mountpoint"}
                            :role "test-role"}
                           {:_id "/test-role/zfs/example-zfs-vol"
                            :label "example-zfs-vol"
                            :name "tank/example/volume"
                            :properties {:mountpoint "none"}
                            :role "test-role"
                            :size "10G"}]}
          :remove @{:zfs @[{:_id "/test-role/zfs/tank_old_filesystem"
                            :name "tank/old/filesystem"
                            :role "test-role"}]}}))

(deftest zfs-error
  (test-error
    (zfs/ensure "pool/fs"
                :properties {:mountpoint "none"}
                :volume-size "100M")
    "In zfs/ensure pool/fs: unexpected property :volume-size. Valid properties are :properties, :size, :label"))
