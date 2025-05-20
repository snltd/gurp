(defn find-named-fn
  "When given a string, returns the function with that name. Or an error"
  [fn-name-as-string]
  (def fn-entry (get (curenv) (symbol fn-name-as-string)))
  (if-not fn-entry
    (error (string/format "'%s' is not defined in this environment" fn-name-as-string)))
  (let [fn-reference (if (table? fn-entry) (get fn-entry :value) fn-entry)]
    (if-not (function? fn-reference)
      (error (string/format "'%s' is not callable" fn-name-as-string)))
    fn-reference))

(defmacro host [host-name & host-definition]
  "The top-level wrapper used to define a host to be configured"
  ~(defn machine-config
     []
     (var resources (array))
     (each host-role (get (table ,;host-definition) :roles)
       (let [fn-to-call (find-named-fn (string host-role "/" host-role))]
         (array/concat resources (fn-to-call))))
     resources))

(defmacro role [role-name & role-definition]
  ~(defn ,role-name
     []
     (var resources (array))
     (each [resource-type resource-spec] (partition 2 (array ,;role-definition))
       (let [fn-to-call (find-named-fn resource-type)]
         (array/push resources (fn-to-call ',role-name resource-spec))))
     resources))

(def- default-protos
  {:file {:owner "root"
          :group "root"}
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
