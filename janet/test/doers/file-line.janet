(use judge)
(use ./lib/test-lib)
(import ../../src/doers/file-line)
(use ../../src/collector)

(deftest file-line
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (import-tests "file-line")

  (test *collector*
    @{:ensure @{:file-line @[{:_id "/test-role/file-line/_path_to_file_1"
                              :line "i-want-to-see-this"
                              :name "/path/to/file_1"
                              :role "test-role"}]}
      :remove @{:file-line @[{:_id "/test-role/file-line/_path_to_file_2"
                              :apply-to "last"
                              :match "starts-with"
                              :name "/path/to/file_2"
                              :pattern "string-prefix"
                              :role "test-role"}
                             {:_id "/test-role/file-line/_path_to_file_3"
                              :apply-to "all"
                              :match "exact"
                              :name "/path/to/file_3"
                              :pattern "i-do-not-want-to-see-this-anywhere"
                              :role "test-role"}
                             {:_id "/test-role/file-line/_path_to_file_4"
                              :apply-to "all"
                              :match "regex"
                              :name "/path/to/file_4"
                              :pattern "rust-regex"
                              :role "test-role"}]}}))

(deftest file-line-error
  (test-error
    (file-line/ensure "/missing/line"
                      :line "and"
                      :after "gibbus"
                      :before "chubb")
    "In file-line/ensure /missing/line: unexpected property :before. Valid properties are :apply-to, :insert-at, :label, :line, :replace, :with")

  (test-error
    (file-line/remove "/my/file"
                      :pattern "merp"
                      :match "end")
    "In file-line/remove /my/file: match must be one of \"exact\", \"starts-with\", \"ends-with\", \"contains\", \"regex\" [Got '\"end\"']"))
