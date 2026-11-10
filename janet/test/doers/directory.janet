(use judge)
(use ./lib/test-lib)
(import ../../src/doers/directory)
(use ../../src/collector)
(import ../../src/doers/file)

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
                             {:_id "/test-role/directory/_example_dir_4"
                              :checksum @{:algorithm "sha512" :suffix "sha512"}
                              :create-from "http://example.com/files/archive.tar"
                              :group "root"
                              :mode "0755"
                              :name "/example/dir_4"
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
    (directory/ensure "/conflicts"
                      :create-from "archive.tar"
                      :create-with "other-archive.tar")
    "In directory/ensure /conflicts: need at most one of :create-from, :create-with")

  (test-error
    (directory/ensure "/extra/keys"
                      :owner "me"
                      :gid 234
                      :recursive true)
    "In directory/ensure /extra/keys: unexpected property :recursive. Valid properties are :checksum, :create-from, :create-with, :group, :label, :mode, :owner"))
