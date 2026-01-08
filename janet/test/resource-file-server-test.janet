(use judge)
(use ../lib/gurp)

# In server mode local file references get turned into http ones, pointing
# to the server.
# 
(deftest "file-resource-server"
  (setdyn :gurp-config-root "/gurpdir")
  (setdyn :role-dyn "test-role")
  (setdyn :server-name "test-server")
  (set *collector* (new-collector))

  (file/ensure "/path/to/file"
               :group "daemon"
               :mode "0755"
               :from "file-test/does-not-exist")

  # (file/ensure "/file/path"
  #              :owner "dataperson"
  #              :mode "0600"
  #              :content "lots-of-data")

  # (file/ensure "/file/from/remote/path"
  #              :owner "gibbus"
  #              :mode "0640"
  #              :with-checksum "0123456789abcdef"
  #              :from-url "https://example.com/files/config")

  (file/remove "/path/to/file")

  (test *collector*
    @{:ensure @{:file @[{:_id "/test-role/file/_path_to_file"
                         :from-url "http://test-server/file/file-test/does-not-exist"
                         :group "daemon"
                         :mode "0755"
                         :name "/path/to/file"
                         :owner "root"
                         :role "test-role"}]}
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

