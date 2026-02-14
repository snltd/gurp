(use judge)
(use ./test-lib)
(use ../../src/collector)
(import ../../src/doers/misc)

(deftest misc
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (import-tests "misc" (curenv))
  (misc/ensure :enable-smb "klf")
  (misc/ensure :enable-smb "frances")

  (test *collector*
    @{:ensure @{:misc @[{:_id "/test-role/misc/nfs-domain-lan.id264.net"
                         :name "nfs-domain-lan.id264.net"
                         :nfs-domain "lan.id264.net"
                         :role "test-role"}
                        {:_id "/test-role/misc/enable-smb-rob"
                         :enable-smb "rob"
                         :name "enable-smb-rob"
                         :role "test-role"}
                        {:_id "/test-role/misc/scheduler-FSS"
                         :name "scheduler-FSS"
                         :role "test-role"
                         :scheduler "FSS"}
                        {:_id "/test-role/misc/enable-smb-klf"
                         :enable-smb "klf"
                         :name "enable-smb-klf"
                         :role "test-role"}
                        {:_id "/test-role/misc/enable-smb-frances"
                         :enable-smb "frances"
                         :name "enable-smb-frances"
                         :role "test-role"}]}
      :remove @{}}))

(deftest misc-error
  (test-error
    (misc/ensure
      :scheduler-class "FSS"
      :enable-smb "rob")
    "unexpected property :scheduler-class. Valid properties are :scheduler, :nfs-domain, :enable-smb, :label"))
