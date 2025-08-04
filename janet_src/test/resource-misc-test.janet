(use judge)
(use ../lib/gurp)

(deftest "test misc functions"
  (setdyn :role-dyn "test-role")
  (test
    (misc/ensure
      :nfs-domain "lan.id264.net")
    {:misc {:_id "/test-role/misc/GENERIC"
            :action :ensure
            :name "GENERIC"
            :nfs-domain "lan.id264.net"
            :role "test-role"}})

  (test
    (misc/ensure
      :scheduler "FSS"
      :enable-smb "rob")
    {:misc {:_id "/test-role/misc/GENERIC"
            :action :ensure
            :enable-smb "rob"
            :name "GENERIC"
            :role "test-role"
            :scheduler "FSS"}})

  (test-error
    (misc/ensure
      :scheduler-class "FSS"
      :enable-smb "rob")
    "Failed to validate user input for misc 'GENERIC' : misc 'GENERIC' has unrecognised key(s): scheduler-class"))
