(defmacro host [host-name & host-definition]
  ~(defn machine-config
  []
     (var resources (array))
     (each host-role (get (table ,;host-definition) :roles)
       (def role-fn (symbol (string host-role "/" host-role)))
       (def fn-entry (get (curenv) role-fn))
       (when (nil? fn-entry)
         (error (string "No such role: " role-fn)))
       (def fn-to-call
         (if (table? fn-entry)
           (get fn-entry :value)
           fn-entry))
       (when (not (function? fn-to-call))
         (error (string "Role is not callable: " role-fn)))
       (array/concat resources (fn-to-call)))
     resources))

(defmacro role [role-name & role-definition]
  ~(defn ,role-name
     []
     (var resources (array))
     (each [resource-type resource-spec] (partition 2 (array ,;role-definition))
       (var resource-type-fn (symbol (string "gurp/" resource-type)))
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

(def- default-protos
  {:file {:owner "root"
          :group "root"}
   # :package {:a 1}
   :directory {:owner "root"
               :group "root"
               :recurse false}})

# For now this is a shim around the hardcoded fallbacks. In the future we'll
# let the user supply their own.
(defn- proto
  "Retrieve resource default values for use as table protos"
  [resource-type]
  (get default-protos resource-type {}))

(defn- resource-from
  [resource-type role-name resource-spec]
  (def user-def
    (struct/with-proto
      (proto resource-type)
      :type resource-type
      :_id (string "/" role-name "/" resource-type "/" (get resource-spec :name))))
  (->>
    (merge resource-spec)
    (merge (struct/proto-flatten user-def))
    (table/to-struct)))

(defn package
  "Given a role and a package name, return a package resource struct. In
  OmniOS, the package version is effectively part of the name"
  [role-name package-name]
  (->>
    (struct/with-proto
      (proto :package)
      :type :package
      :name package-name
      :_id (string "/" role-name "/package/" package-name))
    (struct/proto-flatten)))

(defn file
  "Given a specification, produce a file resource"
  [role-name resource-spec]
  (resource-from :file role-name resource-spec))

(defn directory
  "Given a specification, produce a directory resource"
  [role-name resource-spec]
  (resource-from :directory role-name resource-spec))
