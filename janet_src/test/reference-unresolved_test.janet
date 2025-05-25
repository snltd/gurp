(use judge)
(use ../lib/gurp)

# The reference points to something which does not exist, so should error

(role role-a
      (package/ensure "helix")
      (file/ensure "a1"
                   :path "/tmp/a1"
                   :owner "tester"
                   :content "blah"))
(role role-b
      (file/ensure "b1"
                   :path "/tmp/b1"
                   :owner :/role-a/file/a2/owner
                   :content "blah-blah"))

(host "broken-references"
      (role-a)
      (role-b))

(deftest "missing-reference-should-error"
  (test-error (machine-config) "Referenced resource '/role-a/file/a2' not found"))
