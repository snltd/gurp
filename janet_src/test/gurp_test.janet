(use judge)
(import ../lib/gurp)

(test-macro
  (gurp/host "example-node" :roles ["basenode"])
  (defn machine-config
    []
    (var resources (array))
    (each host-role
      (get (table :roles ["basenode"]) :roles)
      (def role-fn (symbol (string host-role "/" host-role)))
      (def fn-entry (get (curenv) role-fn))
      (when (nil? fn-entry)
        (error (string "No such role: " role-fn)))
      (def fn-to-call (if (table? fn-entry) (get fn-entry :value) fn-entry))
      (when (not (function? fn-to-call))
        (error (string "Role is not callable: " role-fn)))
      (array/concat resources (fn-to-call)))
    resources))

(test-macro
  (gurp/role "basenode" :package "helix" :package "rust")
  (defn "basenode"
    []
    (var resources (array))
    (each [resource-type resource-spec]
      (partition 2 (array :package "helix" :package "rust"))
      (var resource-type-fn (symbol (string "gurp/" resource-type)))
      (def fn-entry (get (curenv) resource-type-fn))
      (when (nil? fn-entry)
        (error (string "resource not supported: " resource-type)))
      (def fn-to-call (if (table? fn-entry) (get fn-entry :value) fn-entry))
      (when (not (function? fn-to-call))
        (error (string "resource fn is not callable: " resource-type)))
      (array/push resources (fn-to-call (quote "basenode") resource-spec)))
    resources))

(deftest "role"
  (gurp/role
    example-role
    :package "helix"
    :package "chubb")
  (test (example-role)
    @[{:_id "/example-role/package/helix"
       :name "helix"
       :type :package}
      {:_id "/example-role/package/chubb"
       :name "chubb"
       :type :package}]))

(deftest "create-package-resource"
  (test (gurp/package "test-role" "helix")
    {:_id "/test-role/package/helix"
     :name "helix"
     :type :package}))

(deftest "create-file-resource"
  (test (gurp/file "test-role"
                      {:name "my-file"
                       :path "/my/file"
                       :owner "rob"})
    {:_id "/test-role/file/my-file"
     :group "root"
     :name "my-file"
     :owner "rob"
     :path "/my/file"
     :type :file}))

(deftest "create-directory-resource"
  (test (gurp/directory "test-role"
                           {:name "my-dir"
                            :path "/my/dir"
                            :group "sysadmin"})
    {:_id "/test-role/directory/my-dir"
     :group "sysadmin"
     :name "my-dir"
     :owner "root"
     :path "/my/dir"
     :recurse false
     :type :directory}))
