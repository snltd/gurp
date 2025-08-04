(use judge)
(use ../lib/gurp)

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
    "Failed to validate user input for symlink '/where/does/this/point' : symlink missing required key(s): source")

  (test-error
    (symlink/ensure "/symlinks/dont/work/like/that"
                    :source "/some/file"
                    :owner "me")
    "Failed to validate user input for symlink '/symlinks/dont/work/like/that' : symlink '/symlinks/dont/work/like/that' has unrecognised key(s): owner"))
