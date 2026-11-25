(use judge)
(use ./lib/test-lib)
(import ../../src/doers/file)
(use ../../src/collector)

(deftest file
  (setdyn :gurp-config-root "/gurpdir")
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (import-tests "file")

  (test *collector*
        @{:ensure @{:file @[{:_id "/test-role/file/_example_file_from-content"
                             :content "words\n and\nstuff\n"
                             :group "root"
                             :mode "0600"
                             :name "/example/file/from-content"
                             :owner "sys"
                             :role "test-role"}
                            {:_id "/test-role/file/_example_file_from-local-file"
                             :from "/gurpdir/files/file-dir/example"
                             :group "daemon"
                             :mode "4755"
                             :name "/example/file/from-local-file"
                             :owner "root"
                             :role "test-role"}
                            {:_id "/test-role/file/remote-file"
                             :checksum @{:algorithm "sha256"
                                         :literal "7bde5db38472e2c187ad2e5e10c25a4aecb59e8304c26b9894d11c4b99196ea7"}
                             :from-url "https://raw.githubusercontent.com/snltd/gurp/refs/heads/main/LICENSE.txt"
                             :group "root"
                             :label "remote-file"
                             :mode "0644"
                             :name "/example/file/from-url"
                             :owner "root"
                             :role "test-role"}]}
          :remove @{:file @[{:_id "/test-role/file/_path_to_file"
                             :name "/path/to/file"
                             :role "test-role"}]}}))

(deftest relative-without-server
  (test
    (file/ensure "/target/path" :from "relative-path")
    {:_id "/test-role/file/_target_path"
     :from "/gurpdir/files/relative-path"
     :group "root"
     :mode "0644"
     :name "/target/path"
     :owner "root"
     :role "test-role"}))

(deftest relative-with-server
  (setdyn :server-name "test-server")
  (setdyn :gurp-config-root "/gurpdir")
  (test
    (file/ensure "/target/path" :from "relative-path")
    {:_id "/test-role/file/_target_path"
     :from-url "http://test-server/v1/file/relative-path"
     :group "root"
     :mode "0644"
     :name "/target/path"
     :owner "root"
     :role "test-role"
     :url-is-server true})
  (setdyn :server-name nil)
  (setdyn :gurp-config-root nil))

(deftest qualified-without-server
  (test
    (file/ensure "/target/path" :from "/fully/qualified-path")
    {:_id "/test-role/file/_target_path"
     :from "/fully/qualified-path"
     :group "root"
     :mode "0644"
     :name "/target/path"
     :owner "root"
     :role "test-role"}))

(deftest qualified-with-server
  (setdyn :server-name "test-server")
  (setdyn :gurp-config-root "/gurpdir")
  (test
    (file/ensure "/target/path" :from "/fully/qualified-path")
    {:_id "/test-role/file/_target_path"
     :from "/fully/qualified-path"
     :group "root"
     :mode "0644"
     :name "/target/path"
     :owner "root"
     :role "test-role"})
  (setdyn :server-name nil)
  (setdyn :gurp-config-root nil))

(deftest file-error
  (setdyn :gurp-config-root "/gurpdir")
  (test-error
    (file/ensure "/octals/only"
                 :content "bleurgh"
                 :owner "merp"
                 :group "byerp"
                 :permissions "rwxr-xr-x")
    "In file/ensure /octals/only: unexpected property :permissions. Valid properties are :backup-suffix, :checksum, :content, :from, :from-struct, :from-url, :group, :ignore-pattern, :label, :mode, :only-fetch-from-url-once, :owner, :to-format, :url-is-server, :url-replacements")

  (test-error
    (file/ensure "/too/many/sources"
                 :from "a-file"
                 :content "blah blah")
    "In file/ensure /too/many/sources: need exactly one of :content, :from, :from-url, :from-struct"))

# In server mode local file references get turned into http ones, pointing to
# the server.
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
               :from-url "https://example.com/files/config"
               (file/checksum
                 :algorithm "sha256"
                 :literal "0123456789abcdef"))

  (file/remove "/path/to/file")

  (test *collector*
        @{:ensure @{:file @[{:_id "/test-role/file/_path_to_file"
                             :from-url "http://test-server/v1/file/file-test/does-not-exist"
                             :group "daemon"
                             :mode "0755"
                             :name "/path/to/file"
                             :owner "root"
                             :role "test-role"
                             :url-is-server true}
                            {:_id "/test-role/file/_file_path"
                             :content "lots-of-data"
                             :group "root"
                             :mode "0600"
                             :name "/file/path"
                             :owner "dataperson"
                             :role "test-role"}
                            {:_id "/test-role/file/_file_from_remote_path"
                             :checksum @{:algorithm "sha256"
                                         :literal "0123456789abcdef"}
                             :from-url "https://example.com/files/config"
                             :group "root"
                             :mode "0640"
                             :name "/file/from/remote/path"
                             :owner "gibbus"
                             :role "test-role"}]}
          :remove @{:file @[{:_id "/test-role/file/_path_to_file"
                             :name "/path/to/file"
                             :role "test-role"}]}}))
