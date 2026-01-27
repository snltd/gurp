(use judge)
(use ./_helpers)
(import ../../src/doers/directory)
(use ../../src/collector)

(deftest directory
  (set *collector* (new-collector))
  (setdyn :role-dyn "test-role")

  (import-tests "directory" (curenv))

  (test *collector*
    @{:ensure @{:directory @[{:_id "/test-role/directory/_path_to_dir_1"
                              :group "root"
                              :mode "0755"
                              :name "/path/to/dir_1"
                              :owner "root"
                              :role "test-role"}
                             {:_id "/test-role/directory/my-dir"
                              :group "root"
                              :label "my-dir"
                              :mode "0700"
                              :name "/path/to/dir_2"
                              :owner "root"
                              :role "test-role"}
                             {:_id "/test-role/directory/all-the-specs"
                              :group "sysadmin"
                              :label "all-the-specs"
                              :mode "0700"
                              :name "/path/to/dir_3"
                              :owner "myself"
                              :role "test-role"}]}
      :remove @{:directory @[{:_id "/test-role/directory/_path_to_dir"
                              :name "/path/to/dir"
                              :role "test-role"}]}}))

(deftest directory-error
  (test-error
    (directory/ensure "/extra/keys"
                      :owner "me"
                      :gid 234
                      :recursive true)
    "unexpected property :recursive. Valid properties are :owner, :group, :mode, :label"))
