(use judge)
(use ../lib/gurp)

(set *collector* (new-collector))

# The reference points to something which does not exist, so should error

(role role-a
      (pkg/ensure "helix")
      (file/ensure "/tmp/a1"
                   :label "a1"
                   :owner "tester"
                   :content "blah"))
(role role-b
      (file/ensure "/tmp/b1"
                   :owner :/role-a/file/a2/owner
                   :content "blah-blah"))

(host "broken-references"
      (role-a)
      (role-b))

(deftest "missing-reference-should-error"
  (test-error (machine-config) "Referenced resource '/role-a/file/a2' not found"))
