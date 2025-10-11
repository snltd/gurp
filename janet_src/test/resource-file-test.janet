(use judge)
(use ../lib/gurp)

(deftest "file-resource"
  (setdyn :gurp-config-root "/gurpdir")
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (file/ensure "/path/to/file"
               :group "daemon"
               :mode "0755"
               :from "file-test/does-not-exist")

  (file/ensure "/file/path"
               :owner "dataperson"
               :mode "0600"
               :content "lots-of-data")

  (file/ensure "/secret/file"
               :owner "root"
               :mode "0600"
               :content "sensitive-data"
               :url-replacements {
                "__SENSITIVE_VALUE_1__" "https://secret-server/secrets/value_1"
              }
              )

  (file/ensure "/file/from/remote/path"
               :owner "gibbus"
               :mode "0640"
               :with-checksum "0123456789abcdef"
               :from-url "https://example.com/files/config")

  (file/remove "/path/to/file")

  (test *collector*
    @{:ensure @{:file @[{:_id "/test-role/file/_path_to_file"
                         :from "/gurpdir/files/file-test/does-not-exist"
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

(deftest "file-error"
  (test-error
    (file/ensure "/octals/only"
                 :owner "merp"
                 :group "byerp"
                 :permissions "rwxr-xr-x")
    "Failed to validate user input for file '/octals/only': file '/octals/only' has unrecognised key(s): permissions"))

