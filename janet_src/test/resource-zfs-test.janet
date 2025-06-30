(use judge)
(use ../lib/gurp)

(deftest "test zfs functions"
  (setdyn :role-dyn "test-role")
  (test
    (zfs/ensure (zfscat "tank" "export" "test")
                    :label "test-zfs"
                    :compression "gzip9"
                    :devices "off")
    {:zfs {:_id "/test-role/zfs/test-zfs"
           :action :ensure
           :compression "gzip9"
           :devices "off"
           :label "test-zfs"
           :name "tank/export/test"
           :role "test-role"}})
  (test
    (zfs/remove "old/filesystem")
    {:zfs {:_id "/test-role/zfs/old_filesystem"
           :action :remove
           :name "old/filesystem"
           :role "test-role"}}))
