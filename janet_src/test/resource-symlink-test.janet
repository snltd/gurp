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
  (test
    (symlink/remove "/dont/want/this/link")
    {:symlink {:_id "/test-role/symlink/_dont_want_this_link"
               :action :remove
               :name "/dont/want/this/link"
               :role "test-role"}}))
