(use judge)
(use ./test-lib)
(use ../../src/collector)
(use ../../src/dsl)
(import ../../src/doers/link)

(deftest link
  (set *collector* (new-collector))
  (setdyn :role-dyn "test-role")

  (import-tests "link")

  (test *collector*
    @{:ensure @{:link @[{:_id "/test-role/link/_link_is_here"
                         :force-link false
                         :name "/link/is/here"
                         :role "test-role"
                         :source "/link/points/here"
                         :type "hard"}
                        {:_id "/test-role/link/example-symlink"
                         :force-link true
                         :label "example-symlink"
                         :name "/symlink/is/here"
                         :role "test-role"
                         :source "/link/points/here"
                         :type "symbolic"}]}
      :remove @{:link @[{:_id "/test-role/link/_dont_want_this_link"
                         :name "/dont/want/this/link"
                         :role "test-role"}]}}))

(deftest link-error
  (test-error
    (link/ensure "/where/does/this/point")
    "In link/ensure /where/does/this/point: did not find mandatory property :source. Mandatory properties are :force-link, :source, :type")

  (test-error
    (link/ensure "/links/dont/work/like/that"
                 :source "/some/file"
                 :owner "me")
    "In link/ensure /links/dont/work/like/that: unexpected property :owner. Valid properties are :force-link, :source, :type, :label"))
