(use judge)
(use ./test-lib)
(import ../../src/doers/file)
(use ../../src/collector)

(deftest file
  (setdyn :gurp-config-root "/gurpdir")
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (import-tests "file" (curenv))

  (test *collector*
        @{:ensure @{:file @[{:_id "/test-role/file/_file_from_local_file"
                             :from "/gurpdir/files/file-test/does-not-exist"
                             :group "daemon"
                             :mode "0755"
                             :name "/file/from/local_file"
                             :owner "root"
                             :role "test-role"}
                            {:_id "/test-role/file/_file_from_content"
                             :content "lots-of-data"
                             :group "root"
                             :mode "0600"
                             :name "/file/from/content"
                             :owner "dataperson"
                             :role "test-role"}
                            {:_id "/test-role/file/remote-file"
                             :from-url "https://example.com/files/config"
                             :group "root"
                             :label "remote-file"
                             :mode "0640"
                             :name "/file/from/arbitrary/server"
                             :owner "gibbus"
                             :role "test-role"
                             :with-checksum "0123456789abcdef"}]}
          :remove @{:file @[{:_id "/test-role/file/_path_to_file"
                             :name "/path/to/file"
                             :role "test-role"}]}}))

(deftest file-error
  (test-error
    (file/ensure "/octals/only"
                 :owner "merp"
                 :group "byerp"
                 :permissions "rwxr-xr-x")
    "In file/ensure /octals/only: unexpected property :permissions. Valid properties are :owner, :content, :from-url, :group, :mode, :from-struct, :with-checksum, :from, :ignore-pattern, :to-format, :backup-suffix, :label"))

# In server mode local file references get turned into http ones, pointing
# to the server.
# 
(deftest file-resource-server
  (setdyn :gurp-config-root "/gurpdir")
  (setdyn :role-dyn "test-role")
  (setdyn :server-name "test-server")
  (set *collector* (new-collector))

  (file/ensure "/path/to/file"
               :group "daemon"
               :mode "0755"
               :from "file-test/does-not-exist")

  (file/ensure "/file/path"
               :owner "dataperson"
               :mode "0600"
               :content "lots-of-data")

  (file/ensure "/file/from/remote/path"
               :owner "gibbus"
               :mode "0640"
               :with-checksum "0123456789abcdef"
               :from-url "https://example.com/files/config")

  (file/remove "/path/to/file")

  (test *collector*
    @{:ensure @{:file @[{:_id "/test-role/file/_path_to_file"
                         :from-url "http://test-server/v1/file/file-test/does-not-exist"
                         :group "daemon"
                         :mode "0755"
                         :name "/path/to/file"
                         :owner "root"
                         :role "test-role"}
                        {:_id "/test-role/file/_file_path"
                         :content "lots-of-data"
                         :group "root"
                         :mode "0600"
                         :name "/file/path"
                         :owner "dataperson"
                         :role "test-role"}
                        {:_id "/test-role/file/_file_from_remote_path"
                         :from-url "https://example.com/files/config"
                         :group "root"
                         :mode "0640"
                         :name "/file/from/remote/path"
                         :owner "gibbus"
                         :role "test-role"
                         :with-checksum "0123456789abcdef"}]}
      :remove @{:file @[{:_id "/test-role/file/_path_to_file"
                         :name "/path/to/file"
                         :role "test-role"}]}}))
