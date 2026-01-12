(use judge)
(import ../../src/doers/file-line)
(use ../../src/collector)

(deftest "test file-line functions"
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (file-line/remove "/tmp/.tmpjpqQir/test-file"
    :pattern "line_2"
    :match "exact"
    :apply-to "all" )

  (file-line/ensure "/path/to/file"
                    :line "i-want-to-see-this")

  (file-line/remove "/path/to/file"
                    :pattern "this-is-an-awful-line")

  (file-line/remove "/path/to/file"
                    :match "exact"
                    :apply-to "last"
                    :pattern "this-is-an-awful-line")

  (test *collector*
    @{:ensure @{:file-line @[@{:_id "/test-role/file-line/_path_to_file"
                               :line "i-want-to-see-this"
                               :name "/path/to/file"
                               :role "test-role"}]}
      :remove @{:file-line @[@{:_id "/test-role/file-line/_tmp_.tmpjpqQir_test-file"
                               :apply-to "all"
                               :match "exact"
                               :name "/tmp/.tmpjpqQir/test-file"
                               :pattern "line_2"
                               :role "test-role"}
                             @{:_id "/test-role/file-line/_path_to_file"
                               :apply-to "all"
                               :match "exact"
                               :name "/path/to/file"
                               :pattern "this-is-an-awful-line"
                               :role "test-role"}
                             @{:_id "/test-role/file-line/_path_to_file"
                               :apply-to "last"
                               :match "exact"
                               :name "/path/to/file"
                               :pattern "this-is-an-awful-line"
                               :role "test-role"}]}}))

(deftest "file-line-error"
  (test-error
    (file-line/ensure "/missing/line"
                      :line "and"
                      :after "gibbus"
                      :before "chubb")
    "unexpected property :before. Valid properties are: :with, :apply-to, :replace, :label, :insert-at, :line")

  (test-error
    (file-line/remove "/my/file"
                      :pattern "merp"
                      :match "end")
    "match must be one of \"exact\", \"starts_with\", \"ends_with\", \"contains\", \"matches\" [Got 'end']"))

