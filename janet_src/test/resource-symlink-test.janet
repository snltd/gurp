(use judge)
(use ../lib/gurp)

(deftest "test symlink functions"
  (setdyn :role-dyn "test-role")
  (test
    (symlink/ensure (pathcat "link" "is" "here")
                    :label "test-link"
                    :source "/link/points/here")
    {:symlink {:_id "/test-role/symlink/test-link"
               :action :ensure
               :label "test-link"
               :name "/link/is/here"
               :role "test-role"
               :source "/link/points/here"}})
  (test-error
    (symlink/ensure "/where/does/this/point")
    "Failed to validate user input for symlink '/where/does/this/point' : symlink missing required key(s): source")

  (test-error
    (symlink/ensure "/symlinks/dont/work/like/that"
                    :source "/some/file"
                    :owner "me")
    "Failed to validate user input for symlink '/symlinks/dont/work/like/that' : symlink '/symlinks/dont/work/like/that' has unrecognised key(s): owner")

  (test
    (symlink/remove "/dont/want/this/link")
    {:symlink {:_id "/test-role/symlink/_dont_want_this_link"
               :action :remove
               :name "/dont/want/this/link"
               :role "test-role"}}))
