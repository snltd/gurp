(use judge)
(use ../lib/gurp)

(deftest "test file-line functions"
  (setdyn :role-dyn "test-role")
  (test
    (file-line/ensure "/path/to/file"
                      :line "i-want-to-see-this")
    {:file-line {:_id "/test-role/file-line/_path_to_file"
                 :action :ensure
                 :line "i-want-to-see-this"
                 :name "/path/to/file"
                 :role "test-role"}})
  (test
    (file-line/remove "/path/to/file"
                      :line "this-is-an-awful-line")
    {:file-line {:_id "/test-role/file-line/_path_to_file"
                 :action :remove
                 :line "this-is-an-awful-line"
                 :name "/path/to/file"
                 :role "test-role"}}))
