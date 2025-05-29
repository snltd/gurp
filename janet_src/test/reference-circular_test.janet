(use judge)
(use ../lib/gurp)

# The references are circular, so should cause an error

(role role-a
      (pkg/ensure "helix")
      (file/ensure "a1"
                   :path "/tmp/a1"
                   :owner :/role-b/file/b2/owner
                   :content "blah"))
(role role-b
      (file/ensure "b1"
                   :path "/tmp/b1"
                   :owner :/role-a/file/a1/owner
                   :content "blah-blah")
      (file/ensure "b2"
                   :path "/tmp/b2"
                   :owner :/role-b/file/b1/owner
                   :content "blah-blah-blah"))

(host "circular-dependency"
      (role-a)
      (role-b))

(deftest "circular-reference-should-error"
  (test-error (machine-config)
    "Detected circular reference [@[\"/role-b/file/b2\" \"/role-b/file/b1\" \"/role-a/file/a1\"]]"))  
