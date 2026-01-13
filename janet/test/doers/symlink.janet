(use judge)
(use ../../src/collector)
(use ../../src/user-helpers)
(import ../../src/doers/symlink)

(deftest "symlink-resources"
  (set *collector* (new-collector))

  (setdyn :role-dyn "test-role")

  (symlink/ensure (pathcat "link" "is" "here")
                  :label "test-link"
                  :source "/link/points/here")

  (symlink/remove "/dont/want/this/link")

  (test *collector*
        @{:ensure @{:symlink @[{:_id "/test-role/symlink/test-link"
                                :label "test-link"
                                :name "/link/is/here"
                                :role "test-role"
                                :source "/link/points/here"}]}
          :remove @{:symlink @[{:_id "/test-role/symlink/_dont_want_this_link"
                                :name "/dont/want/this/link"
                                :role "test-role"}]}}))

(deftest "symlink-resources"
  (test-error
    (symlink/ensure "/where/does/this/point")
    "did not find mandatory property :source. Mandatory properties are :source")

  (test-error
    (symlink/ensure "/symlinks/dont/work/like/that"
                    :source "/some/file"
                    :owner "me")
    "unexpected property :owner. Valid properties are :source, :label"))
