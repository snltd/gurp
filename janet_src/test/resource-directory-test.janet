(use judge)
(use ../lib/gurp)

(deftest "test directory functions"
  (setdyn :role-dyn "test-role")
  (test
    (directory/ensure "/path/to/dir"
                      :label "my-dir"
                      :mode "0700")
    {:directory {:_id "/test-role/directory/my-dir"
                 :action :ensure
                 :group "root"
                 :label "my-dir"
                 :mode "0700"
                 :name "/path/to/dir"
                 :owner "root"
                 :role "test-role"}})

  (test
    (directory/ensure "/path/to/default/dir")
    {:directory {:_id "/test-role/directory/_path_to_default_dir"
                 :action :ensure
                 :group "root"
                 :mode "0755"
                 :name "/path/to/default/dir"
                 :owner "root"
                 :role "test-role"}})

  (test
    (directory/ensure "/highly/specified/dir"
                      :owner "myself"
                      :group "sysadmin"
                      :mode "0700"
                      :label "all-the-specs")
    {:directory {:_id "/test-role/directory/all-the-specs"
                 :action :ensure
                 :group "sysadmin"
                 :label "all-the-specs"
                 :mode "0700"
                 :name "/highly/specified/dir"
                 :owner "myself"
                 :role "test-role"}})

  (test-error
    (directory/ensure "/extra/keys"
                      :owner "me"
                      :gid 234
                      :recursive true)
    "directory '/extra/keys' has unrecognised key(s): recursive, gid")

  (test
    (directory/remove "/path/to/dir")
    {:directory {:_id "/test-role/directory/_path_to_dir"
                 :action :remove
                 :name "/path/to/dir"
                 :role "test-role"}}))
