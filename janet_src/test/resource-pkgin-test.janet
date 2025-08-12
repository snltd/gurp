(use judge)
(use ../lib/gurp)

(deftest "pkgin-resources"
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (pkgin/ensure "rust")
  (pkgin/remove "go")
  (pkgin/remove "python")

  (test *collector*
    @{:ensure @{:pkgin @[{:_id "/test-role/pkgin/rust"
                          :name "rust"
                          :role "test-role"}]}
      :remove @{:pkgin @[{:_id "/test-role/pkgin/go"
                          :name "go"
                          :role "test-role"}
                         {:_id "/test-role/pkgin/python"
                          :name "python"
                          :role "test-role"}]}}))

(deftest "pkgin-error"
  (test-error
    (pkgin/ensure "gurp"
                :version "1.1.1")
    "Failed to validate user input for pkgin 'gurp' : pkgin 'gurp' has unrecognised key(s): version"))

