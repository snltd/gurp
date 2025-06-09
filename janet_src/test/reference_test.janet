(use judge)
(use ../lib/gurp)

# These files should all end up with the same owner

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

(deftest "references-should-all-resolve"
  (test (machine-config)
    {:metadata {:name "circular-dependency"}
     :resources {:ensure {:file @[{:_id "/role-a/file/a1"
                                   :action :ensure
                                   :content "blah"
                                   :group "sysadmin"
                                   :label "a1"
                                   :mode "0644"
                                   :name "/tmp/a1"
                                   :owner "tester"
                                   :role "role-a"}
                                  {:_id "/role-b/file/_tmp_b1"
                                   :action :ensure
                                   :content "blah-blah"
                                   :group "sysadmin"
                                   :mode "0644"
                                   :name "/tmp/b1"
                                   :owner "tester"
                                   :role "role-b"}
                                  {:_id "/role-b/file/b2"
                                   :action :ensure
                                   :content "blah-blah-blah"
                                   :group "root"
                                   :label "b2"
                                   :mode "0644"
                                   :name "/tmp/b2"
                                   :owner "tester"
                                   :role "role-b"}
                                  {:_id "/role-b/file/_tmp_b3"
                                   :action :ensure
                                   :content "blah-blah-blah"
                                   :group "root"
                                   :mode "0644"
                                   :name "/tmp/b3"
                                   :owner "tester"
                                   :role "role-b"}]}}}))
