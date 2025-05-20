(use judge)
(use ../lib/gurp)

(test-macro
  (host "example-node" :roles ["basenode"])
  (defn machine-config
    []
    (var resources (array))
    (each host-role
      (get (table :roles ["basenode"]) :roles)
      (let [fn-to-call (find-named-fn (string host-role "/" host-role))]
        (array/concat resources (fn-to-call))))
    resources))

(test-macro
  (role "basenode" :package "helix" :package "rust")
  (defn "basenode"
    []
    (var resources (array))
    (each [resource-type resource-spec]
      (partition 2 (array :package "helix" :package "rust"))
      (let [fn-to-call (find-named-fn resource-type)]
        (array/push resources (fn-to-call (quote "basenode") resource-spec))))
    resources))

(deftest "role"
  (role
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
  (test (package "test-role" "helix")
        {:_id "/test-role/package/helix"
         :name "helix"
         :type :package}))

(deftest "create-file-resource"
  (test (file "test-role"
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
  (test (directory "test-role"
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

(def not-a-function "fixture for test-find-named-fn" nil)
(defn function-to-be-found "fixture for test-find-named-fn" [] nil)

(deftest "test-find-named-fn"
  (test (find-named-fn "function-to-be-found") @function-to-be-found)
  (test (type (find-named-fn "function-to-be-found")) :function)

  (test-error (find-named-fn "no-such-function")
              "'no-such-function' is not defined in this environment")

  (test-error (find-named-fn "not-a-function")
              "'not-a-function' is not callable"))
