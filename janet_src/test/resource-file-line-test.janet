(use judge)
(use ../lib/gurp)

(deftest "test file-line functions"
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (file-line/ensure "/path/to/file"
                    :line "i-want-to-see-this")

  (file-line/remove "/path/to/file"
                    :line "this-is-an-awful-line")

  (file-line/remove "/path/to/file"
                    :match "last"
                    :line "this-is-an-awful-line")

  (test *collector*
        @{:ensure @{:file-line @[{:_id "/test-role/file-line/_path_to_file"
                                  :line "i-want-to-see-this"
                                  :name "/path/to/file"
                                  :role "test-role"}]}
          :remove @{:file-line @[{:_id "/test-role/file-line/_path_to_file"
                                  :apply-to "all"
                                  :line "this-is-an-awful-line"
                                  :match "exact"
                                  :name "/path/to/file"
                                  :role "test-role"}
                                 {:_id "/test-role/file-line/_path_to_file"
                                  :apply-to "all"
                                  :line "this-is-an-awful-line"
                                  :match "last"
                                  :name "/path/to/file"
                                  :role "test-role"}]}}))

(deftest "file-line-error"
  (test-error
    (file-line/ensure "/missing/line"
                      :line "and"
                      :after "gibbus"
                      :before "chubb")
    "Failed to validate user input for file-line '/missing/line' : file-line '/missing/line' has unrecognised key(s): before, after")

  (test-error
    (file-line/remove "/my/file"
                      :pattern "merp"
                      :match "end")
    "match must be one of all, first, last")

  (test-error
    (file-line/ensure "/missing/line")
    "Failed to validate user input for file-line '/missing/line' : file-line missing required key(s): line"))
