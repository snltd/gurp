(use judge)
(use ./test-lib)
(use ../../src/collector)
(use ../../src/dsl)
(import ../../src/doers/link)

(deftest link
  (set *collector* (new-collector))
  (setdyn :role-dyn "test-role")

  (import-tests "link" (curenv))

  (test *collector*
    @{:ensure @{:link @[{:_id "/test-role/link/example-symlink"
                         :label "example-symlink"
                         :name "/symlink/is/here"
                         :role "test-role"
                         :source "/link/points/here"
                         :type "symbolic"}
                        {:_id "/test-role/link/_link_is_here"
                         :name "/link/is/here"
                         :role "test-role"
                         :source "/link/points/here"
                         :type "hard"}]}
      :remove @{:link @[{:_id "/test-role/link/_dont_want_this_link"
                         :name "/dont/want/this/link"
                         :role "test-role"}]}}))

(deftest link-error
  (test-error
    (link/ensure "/where/does/this/point")
    "did not find mandatory property :source. Mandatory properties are :source")

  (test-error
    (link/ensure "/links/dont/work/like/that"
                    :source "/some/file"
                    :owner "me")
    "unexpected property :owner. Valid properties are :source, :label"))
