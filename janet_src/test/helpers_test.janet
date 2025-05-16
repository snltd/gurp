(use judge)
(import ../lib/helpers)

(test-macro
  (helpers/role "basenode" :package "helix" :package "rust")
  (defn "basenode"
    []
    (var resources (array))
    (each [resource-type resource-spec]
      (partition 2 (array :package "helix" :package "rust"))
      (var resource-type-fn (symbol (string "helpers/" resource-type)))
      (def fn-entry (get (curenv) resource-type-fn))
      (when (nil? fn-entry)
        (error (string "resource not supported: " resource-type)))
      (def fn-to-call (if (table? fn-entry) (get fn-entry :value) fn-entry))
      (when (not (function? fn-to-call))
        (error (string "resource fn is not callable: " resource-type)))
      (array/push resources (fn-to-call (quote "basenode") resource-spec)))
    resources))

(deftest "role"
  (helpers/role
    example-role
    :package "helix"
    :package "chubb")
  (test (example-role)
        @[{:_id "example-role/package/helix"
           :name "helix"
           :type :package}
          {:_id "example-role/package/chubb"
           :name "chubb"
           :type :package}]))

(test (helpers/package "test-role" "helix")
      {:_id "test-role/package/helix"
       :name "helix"
       :type :package})

(test (helpers/file "test-role" {:name "my-file" :owner "rob"})
      {:_id "test-role/file/my-file"
       :name "my-file"
       :owner "rob"
       :group "root"
       :type :file})
