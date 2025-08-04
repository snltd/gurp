(use judge)
(use ../lib/gurp)

(deftest "misc-resources"
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (misc/ensure :nfs-domain "lan.id264.net")

  (misc/ensure
    :scheduler "FSS"
    :enable-smb "rob")

  (test *collector*
        @{:ensure @{:misc @[{:_id "/test-role/misc/GENERIC"
                             :name "GENERIC"
                             :nfs-domain "lan.id264.net"
                             :role "test-role"}
                            {:_id "/test-role/misc/GENERIC"
                             :enable-smb "rob"
                             :name "GENERIC"
                             :role "test-role"
                             :scheduler "FSS"}]}
          :remove @{}}))

(deftest "misc-error"
  (test-error
    (misc/ensure
      :scheduler-class "FSS"
      :enable-smb "rob")
    "Failed to validate user input for misc 'GENERIC' : misc 'GENERIC' has unrecognised key(s): scheduler-class"))
