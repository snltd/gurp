(use judge)
(use ./test-lib)
(import ../../src/doers/directory)
(use ../../src/collector)

(deftest directory
  (set *collector* (new-collector))
  (setdyn :role-dyn "test-role")

  (import-tests "directory")

  (test *collector*
    @{:ensure @{:directory @[{:_id "/test-role/directory/_example_dir_1"
                              :group "root"
                              :mode "0755"
                              :name "/example/dir_1"
                              :owner "root"
                              :role "test-role"}
                             {:_id "/test-role/directory/my-dir"
                              :group 12
                              :label "my-dir"
                              :mode "2750"
                              :name "/example/dir_3"
                              :owner 4
                              :role "test-role"}
                             {:_id "/test-role/directory/all-the-specs"
                              :group "sys"
                              :label "all-the-specs"
                              :mode "0700"
                              :name "/example/dir_2"
                              :owner "adm"
                              :role "test-role"}]}
      :remove @{:directory @[{:_id "/test-role/directory/_example"
                              :name "/example"
                              :role "test-role"}]}}))


(deftest directory-error
  (test-error
    (directory/ensure "/extra/keys"
                      :owner "me"
                      :gid 234
                      :recursive true)
    "In directory/ensure /extra/keys: unexpected property :recursive. Valid properties are :owner, :group, :mode, :label"))
