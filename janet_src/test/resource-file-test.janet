(use judge)
(use ../lib/gurp)

(deftest "test file functions"
  (setdyn :gurp-config-root "/gurpdir")
  (setdyn :role-dyn "test-role")
  (test
    (file/ensure "/path/to/file"
      :group "daemon"
      :mode "0755"
      :from "file-test/does-not-exist")
    {:file {:_id "/test-role/file/_path_to_file"
            :action :ensure
            :from "/gurpdir/files/file-test/does-not-exist"
            :group "daemon"
            :mode "0755"
            :name "/path/to/file"
            :owner "root"
            :role "test-role"}})

  (test
    (file/ensure "/file/path"
      :owner "dataperson"
      :mode "0600"
      :content "lots-of-data")
    {:file {:_id "/test-role/file/_file_path"
            :action :ensure
            :content "lots-of-data"
            :group "root"
            :mode "0600"
            :name "/file/path"
            :owner "dataperson"
            :role "test-role"}})

    (test
    (file/remove "/path/to/file")
      {:file {:_id "/test-role/file/_path_to_file"
              :action :remove
              :name "/path/to/file"
              :role "test-role"}}))
