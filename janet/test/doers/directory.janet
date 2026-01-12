(use judge)
(import ../../src/doers/directory)
(use ../../src/collector)

(deftest "directory-resources"
  (set *collector* (new-collector))
  (setdyn :role-dyn "test-role")

  (directory/ensure "/path/to/dir"
                    :label "my-dir"
                    :mode "0700")

  (directory/ensure "/path/to/default/dir")

  (directory/ensure "/highly/specified/dir"
                    :owner "myself"
                    :group "sysadmin"
                    :mode "0700"
                    :label "all-the-specs")

  (directory/remove "/path/to/dir")

  (test *collector*
    @{:ensure @{:directory @[@{:_id "/test-role/directory/_path_to_dir"
                               :group "root"
                               :label "my-dir"
                               :mode "0700"
                               :name "/path/to/dir"
                               :owner "root"
                               :role "test-role"}
                             @{:_id "/test-role/directory/_path_to_default_dir"
                               :group "root"
                               :mode "0755"
                               :name "/path/to/default/dir"
                               :owner "root"
                               :role "test-role"}
                             @{:_id "/test-role/directory/_highly_specified_dir"
                               :group "sysadmin"
                               :label "all-the-specs"
                               :mode "0700"
                               :name "/highly/specified/dir"
                               :owner "myself"
                               :role "test-role"}]}
      :remove @{:directory @[@{:_id "/test-role/directory/_path_to_dir"
                               :name "/path/to/dir"
                               :role "test-role"}]}}))

(deftest "directory-error"
  (test-error
    (directory/ensure "/extra/keys"
                      :owner "me"
                      :gid 234
                      :recursive true)
    "unexpected property :recursive. Valid properties are: :owner, :group, :mode, :label"))

