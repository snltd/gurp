(use judge
     ./lib/test-lib
     ../../src/collector
     ../../src/dsl)
(import ../../src/doers/zfs)
(import ../../src/doers/file)

(deftest zfs
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (import-tests "zfs")

  (zfs/ensure "rpool/blank")

  (test *collector*
        @{:ensure @{:zfs @[{:_id "/test-role/zfs/rpool_example"
                            :checksum @{:algorithm "sha256"
                                        :literal "--checksum-goes-here--"}
                            :create-from "http://example.com/zfs/example.zfs"
                            :name "rpool/example"
                            :properties @{:atime false
                                          :compression "lz4"
                                          :exec false
                                          :mountpoint "none"
                                          :mtime false}
                            :role "test-role"}
                           {:_id "/test-role/zfs/zfs-example-1"
                            :label "zfs-example-1"
                            :name "rpool/example/filesystem"
                            :properties {:compression "gzip-9"
                                         :dedup true
                                         :devices false
                                         :mountpoint "/example/mountpoint"}
                            :role "test-role"}
                           {:_id "/test-role/zfs/example-zfs-vol"
                            :label "example-zfs-vol"
                            :name "rpool/example/volume"
                            :properties {}
                            :role "test-role"
                            :size "10G"}
                           {:_id "/test-role/zfs/rpool_blank"
                            :name "rpool/blank"
                            :properties @{:mountpoint "none"}
                            :role "test-role"}]}
          :remove @{:zfs @[{:_id "/test-role/zfs/rpool_old_filesystem"
                            :name "rpool/old/filesystem"
                            :role "test-role"}]}}))

(deftest zfs-error
  (test-error
    (zfs/ensure "pool/fs"
                :properties {:mountpoint "none"}
                :volume-size "100M")
    "In zfs/ensure pool/fs: unexpected property :volume-size. Valid properties are :checksum, :create-from, :label, :properties, :size"))
