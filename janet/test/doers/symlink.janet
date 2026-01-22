(use judge)
(use ./_helpers)
(use ../../src/collector)
(use ../../src/user-helpers)
(import ../../src/doers/symlink)

(deftest symlink
  (set *collector* (new-collector))
  (setdyn :role-dyn "test-role")

  (import-tests "symlink" (curenv))

  (test *collector*
        @{:ensure @{:symlink @[{:_id "/test-role/symlink/test-link"
                                :label "test-link"
                                :name "/link/is/here"
                                :role "test-role"
                                :source "/link/points/here"}]}
          :remove @{:symlink @[{:_id "/test-role/symlink/_dont_want_this_link"
                                :name "/dont/want/this/link"
                                :role "test-role"}]}}))

(deftest symlink-error
  (test-error
    (symlink/ensure "/where/does/this/point")
    "did not find mandatory property :source. Mandatory properties are :source")

  (test-error
    (symlink/ensure "/symlinks/dont/work/like/that"
                    :source "/some/file"
                    :owner "me")
    "unexpected property :owner. Valid properties are :source, :label"))
