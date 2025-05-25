(use judge)
(use ../lib/gurp)

# These files should all end up with the same owner

(role role-a
      (file/ensure "a1"
                   :path "/tmp/a1"
                   :group "sysadmin"
                   :owner "tester"
                   :content "blah"))
(role role-b
      (file/ensure "b1"
                   :path "/tmp/b1"
                   :group :/role-a/file/a1/group
                   :owner :/role-a/file/a1/owner
                   :content "blah-blah")
      (file/ensure "b2"
                   :path "/tmp/b2"
                   :owner :/role-a/file/a1/owner
                   :content "blah-blah-blah")
      (file/ensure "b3"
                   :path "/tmp/b3"
                   :owner :/role-b/file/b2/owner
                   :content "blah-blah-blah"))

(host "circular-dependency"
      (role-a)
      (role-b))

(deftest "references-should-all-resolve"
  (test (machine-config)
    {:metadata {:name "circular-dependency"}
     :resources {:ensure {:file @[{:_id "/role-a/file/a1"
                                   :action :ensure
                                   :content "blah"
                                   :group "sysadmin"
                                   :name "a1"
                                   :owner "tester"
                                   :path "/tmp/a1"
                                   :role "role-a"}
                                  {:_id "/role-b/file/b1"
                                   :action :ensure
                                   :content "blah-blah"
                                   :group "sysadmin"
                                   :name "b1"
                                   :owner "tester"
                                   :path "/tmp/b1"
                                   :role "role-b"}
                                  {:_id "/role-b/file/b2"
                                   :action :ensure
                                   :content "blah-blah-blah"
                                   :group "root"
                                   :name "b2"
                                   :owner "tester"
                                   :path "/tmp/b2"
                                   :role "role-b"}
                                  {:_id "/role-b/file/b3"
                                   :action :ensure
                                   :content "blah-blah-blah"
                                   :group "root"
                                   :name "b3"
                                   :owner "tester"
                                   :path "/tmp/b3"
                                   :role "role-b"}]}}}))
