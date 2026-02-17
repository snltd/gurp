(use judge)
(use ./test-lib)
(import ../../src/doers/file-line)
(use ../../src/collector)

(deftest file-line
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (import-tests "file-line" (curenv))

  (test *collector*
    @{:ensure @{:file-line @[{:_id "/test-role/file-line/_path_to_file"
                              :line "i-want-to-see-this"
                              :name "/path/to/file"
                              :role "test-role"}]}
      :remove @{:file-line @[{:_id "/test-role/file-line/_path_to_file"
                              :apply-to "all"
                              :match "exact"
                              :name "/path/to/file"
                              :pattern "i-do-not-want-to-see-this-anywhere"
                              :role "test-role"}
                             {:_id "/test-role/file-line/_path_to_file"
                              :apply-to "all"
                              :match "regex"
                              :name "/path/to/file"
                              :pattern "rust-regex"
                              :role "test-role"}
                             {:_id "/test-role/file-line/_path_to_file"
                              :apply-to "last"
                              :match "starts-with"
                              :name "/path/to/file"
                              :pattern "string-prefix"
                              :role "test-role"}]}})

  (test-error
    (file-line/ensure "/missing/line"
                      :line "and"
                      :after "gibbus"
                      :before "chubb")
    "In file-line/ensure /missing/line: unexpected property :before. Valid properties are :with, :apply-to, :replace, :label, :insert-at, :line")

  (test-error
    (file-line/remove "/my/file"
                      :pattern "merp"
                      :match "end")
    "In file-line/remove /my/file: match must be one of \"exact\", \"starts-with\", \"ends-with\", \"contains\", \"regex\" [Got 'end']"))
