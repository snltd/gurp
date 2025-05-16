(def default-protos
  {:file {:owner "root"
          :group "root"}
   :directory {:owner "root"
               :group "root"
               :recurse false}})


# For now this is a shim around the hardcoded fallbacks. In the future we'll
# let the user supply their own.
(defn proto
  "Retrieve resource default values for use as table protos"
  [resource-type]
  (get default-protos resource-type {}))

(defn package
  "Given a role and a package name, return a package resource struct. In
  OmniOS, the package version is effectively part of the name"
  [role-name package-name]
  (def prototype (proto :package))
  (struct/proto-flatten
    (struct/with-proto {:type :package
                        :name package-name
                        :_id (string role-name "/package/" package-name)})))

(defn file
  [role-name resource-spec]
  (def prototype (proto :file))
  (pp prototype)
  (def user-def
  (struct/with-proto prototype :type :file
                               :_id (string role-name "/file/" (get resource-spec :name))))
  (struct/proto-flatten (table/to-struct (merge resource-spec user-def))))

(defmacro role [role-name & role-definition]
  ~(defn ,role-name
     []
     (var resources (array))
     (each [resource-type resource-spec] (partition 2 (array ,;role-definition))
       (var resource-type-fn (symbol (string "helpers/" resource-type)))
       (def fn-entry (get (curenv) resource-type-fn))
       (when (nil? fn-entry)
         (error (string "resource not supported: " resource-type)))
       (def fn-to-call
         (if (table? fn-entry)
           (get fn-entry :value)
           fn-entry))
       (when (not (function? fn-to-call))
         (error (string "resource fn is not callable: " resource-type)))
       (array/push resources (fn-to-call ',role-name resource-spec)))
     resources))
