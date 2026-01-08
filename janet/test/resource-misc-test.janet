(use judge)
(use ../lib/gurp)

(deftest "misc-resources"
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (misc/ensure :nfs-domain "lan.id264.net")
  (misc/ensure :enable-smb "rob")
  (misc/ensure :enable-smb "klf")
  (misc/ensure :enable-smb "frances")

  (misc/ensure
    :scheduler "FSS"
    :enable-smb "rob")

  (test *collector*
    @{:ensure @{:misc @[{:_id "/test-role/misc/nfs-domain-lan.id264.net"
                         :name "nfs-domain-lan.id264.net"
                         :nfs-domain "lan.id264.net"
                         :role "test-role"}
                        {:_id "/test-role/misc/enable-smb-rob"
                         :enable-smb "rob"
                         :name "enable-smb-rob"
                         :role "test-role"}
                        {:_id "/test-role/misc/enable-smb-klf"
                         :enable-smb "klf"
                         :name "enable-smb-klf"
                         :role "test-role"}
                        {:_id "/test-role/misc/enable-smb-frances"
                         :enable-smb "frances"
                         :name "enable-smb-frances"
                         :role "test-role"}
                        {:_id "/test-role/misc/scheduler-FSS-enable-smb-rob"
                         :enable-smb "rob"
                         :name "scheduler-FSS-enable-smb-rob"
                         :role "test-role"
                         :scheduler "FSS"}]}
      :remove @{}}))

(deftest "misc-error"
  (test-error
    (misc/ensure
      :scheduler-class "FSS"
      :enable-smb "rob")
    "Failed to validate user input for misc 'scheduler-class-FSS-enable-smb-rob': misc 'scheduler-class-FSS-enable-smb-rob' has unrecognised key(s): scheduler-class"))
