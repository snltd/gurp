(use judge)
(use ../../src/collector)
(import ../../src/doers/apk)

(deftest "apk-resources"
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (apk/ensure "rust")
  (apk/remove "go")
  (apk/remove "python")

  (test *collector*
    @{:ensure @{:apk @[{:_id "/test-role/apk/rust"
                        :name "rust"
                        :role "test-role"}]}
      :remove @{:apk @[{:_id "/test-role/apk/go"
                        :name "go"
                        :role "test-role"}
                       {:_id "/test-role/apk/python"
                        :name "python"
                        :role "test-role"}]}}))

(deftest "apk-error"
  (test-error
    (apk/ensure "gurp"
                :version "1.1.1")
    "Failed to validate user input for apk 'gurp': apk 'gurp' has unrecognised key(s): version"))

