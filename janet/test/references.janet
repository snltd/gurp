(use judge)
(use ../src/gurp)

(deftest references-should-all-resolve
  # These files should all end up with the same owner
  (set *collector* (new-collector))
  (role role-a
        (file/ensure "/tmp/a1"
                     :group "sysadmin"
                     :label "a1"
                     :owner "tester"
                     :content "blah")
        (directory/ensure "/tmp/d1"
                          :owner (this :file :a1 :owner)))

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
                     :owner (this :file :b2 :owner)
                     :content "blah-blah-blah"))

  (host "reference-test"
        (role-a)
        (role-b))

  (test (machine-config)
        {:metadata {:name "reference-test"}
         :resources {:ensure @{:directory @[{:_id "/role-a/directory/_tmp_d1"
                                             :group "root"
                                             :mode "0755"
                                             :name "/tmp/d1"
                                             :owner "tester"
                                             :role "role-a"}]
                               :file @[{:_id "/role-a/file/a1"
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

(deftest reference-to-non-existent-resource-should-error
  (set *collector* (new-collector))
  (role role-a
        (file/ensure "/tmp/a1"
                     :label "a1"
                     :owner "tester"
                     :content "blah"))
  (role role-b
        (file/ensure "/tmp/b1"
                     :owner :/role-a/file/a2/owner # a2 is not defined 
                     :content "blah-blah"))

  (host "referenced-resource-does-not-exist"
        (role-a)
        (role-b))

  (test-error
    (machine-config)
    "referenced resource not found: /role-a/file/a2"))

(deftest reference-to-undefined-property-should-error
  (set *collector* (new-collector))
  (role role-a
        (file/ensure "/tmp/a1"
                     :label "a1"
                     :owner "tester"
                     :content "blah"))
  (role role-b
        (file/ensure "/tmp/b1"
                     :mode :/role-a/file/a1/wat
                     :content "blah-blah"))

  (host "referenced-property-is-not-defined"
        (role-a)
        (role-b))

  (test-error
    (machine-config) "referenced property is nil: :/role-a/file/a1/wat"))

(deftest circular-reference-should-error-across-roles
  (set *collector* (new-collector))

  (role role-a
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

(deftest references-in-helpers
  (set *collector* (new-collector))
  (role ref-role
        (directory/ensure "/tmp/d"
                          :label "d1"
                          :owner "tester")

        (smf/ensure "test-service"
                    :description "for reference testing"
                    :fmri (this :directory :d1 :group)
                    (smf/method "start"
                                :exec "/app/method.sh"
                                :user (this :directory :d1 :owner)
                                :group "daemon")))

  (host "helper-refs" (ref-role))
  (test (machine-config)
        {:metadata {:name "helper-refs"}
         :resources {:ensure @{:directory @[{:_id "/ref-role/directory/d1"
                                             :group "root"
                                             :label "d1"
                                             :mode "0755"
                                             :name "/tmp/d"
                                             :owner "tester"
                                             :role "ref-role"}]
                               :smf @[{:_id "/ref-role/smf/test-service"
                                       :default-enabled true
                                       :description "for reference testing"
                                       :fmri "root"
                                       :name "test-service"
                                       :role "ref-role"
                                       :single-instance true
                                       :start-method @{:context {:group "daemon" :user "tester"}
                                                       :exec "/app/method.sh"
                                                       :timeout 60}
                                       :stop-method {:exec ":kill" :timeout 10}}]}
                     :remove @{}}}))
