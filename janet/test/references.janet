(use judge)
(use ../src/gurp)

(comment
(deftest references-should-all-resolve
  # These files should all end up with the same owner
  (set *collector* (new-collector))
  (role role-a
        (file/ensure "/tmp/a1"
                     :group "sysadmin"
                     :label "a1"
                     :owner "tester"
                     :content "blah"))
  (role role-b
        (file/ensure "/tmp/b1"
                     :group :/role-a/file/a1/group
                     :owner :/role-a/file/a1/owner
                     :content "blah-blah")
        (file/ensure "/tmp/b2"
                     :owner :/role-a/file/a1/owner
                     :label "b2"
                     :content "blah-blah-blah")
        (file/ensure "/tmp/b3"
                     :owner :/role-b/file/b2/owner
                     :content "blah-blah-blah"))

  (host "circular-dependency"
        (role-a)
        (role-b))

  (test (machine-config)
        {:metadata {:name "circular-dependency"}
         :resources {:ensure @{:file @[{:_id "/role-a/file/a1"
                                        :content "blah"
                                        :group "sysadmin"
                                        :label "a1"
                                        :mode "0644"
                                        :name "/tmp/a1"
                                        :owner "tester"
                                        :role "role-a"}
                                       {:_id "/role-b/file/_tmp_b1"
                                        :content "blah-blah"
                                        :group "sysadmin"
                                        :mode "0644"
                                        :name "/tmp/b1"
                                        :owner "tester"
                                        :role "role-b"}
                                       {:_id "/role-b/file/b2"
                                        :content "blah-blah-blah"
                                        :group "root"
                                        :label "b2"
                                        :mode "0644"
                                        :name "/tmp/b2"
                                        :owner "tester"
                                        :role "role-b"}
                                       {:_id "/role-b/file/_tmp_b3"
                                        :content "blah-blah-blah"
                                        :group "root"
                                        :mode "0644"
                                        :name "/tmp/b3"
                                        :owner "tester"
                                        :role "role-b"}]}
                     :remove @{}}}))

(deftest dangling-reference-should-error
  (set *collector* (new-collector))
  (role role-a
        (pkg/ensure "helix")
        (file/ensure "/tmp/a1"
                     :label "a1"
                     :owner "tester"
                     :content "blah"))
  (role role-b
        (file/ensure "/tmp/b1"
                     :owner :/role-a/file/a2/owner # a2 is not defined 
                     :content "blah-blah"))

  (host "broken-references"
        (role-a)
        (role-b))

  (test-error
    (machine-config) "Failed to resolve reference '/role-a/file/a2/owner'"))
)

(deftest circular-reference-should-error
  (set *collector* (new-collector))

  (role role-a
        (pkg/ensure "helix")
        (file/ensure "/tmp/a1"
                     :label "a1"
                     :owner :/role-b/file/b1/owner
                     :content "blah"))
  (role role-b
        (file/ensure "/tmp/b1"
                     :label "b1"
                     :owner :/role-a/file/a1/owner
                     :content "blah-blah"))

  (host "circular-dependency"
        (role-a)
        (role-b))

  (test-error
    (machine-config)
    "detected circular reference: [@{\"/role-a/file/a1\" true \"/role-b/file/b1\" true}]"))
