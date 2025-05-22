(def- default-protos
  {:file {:owner "root"
          :group "root"}
   :directory {:owner "root"
               :group "root"
               :recurse false}})

# For now this is a shim around the hardcoded fallbacks. In the future we'll
# let the user supply their own. Not sure how, yet.
(defn- proto
  "Retrieve resource default values for use as table protos"
  [resource-type]
  (get default-protos resource-type {}))

(defn generic-resource
  "Creates a resource struct of the given type and name. The role is picked up
  from a role-specific dynamic binding, to cut down on user boilerplate"
  [resource-type action resource-name resource-spec]
  (->>
    (struct/with-proto
      (proto resource-type)
      :_id (string "/" (dyn :role-dyn) "/" resource-type "/" resource-name)
      :role (dyn :role-dyn)
      :name resource-name
      :action action
      (splice resource-spec))
    (struct/proto-flatten)
    (merge)
    (table/to-struct)
    (struct resource-type)))

(defn package/ensure [name &]
  "Given a a package name, return a package ensure struct. In OmniOS, the
  package version is effectively part of the name, so there are no parameters"
  (generic-resource :package :ensure name []))

(defn package/remove [name &]
  "Given a package name, return a package remove struct"
  (generic-resource :package :remove name []))

(defn file/ensure [name & specs]
  "Given a file name and specification, return a file ensure struct"
  (generic-resource :file :ensure name specs))

(defn file/remove [name & specs]
  "Given a file name and specification, return a file remove struct"
  (generic-resource :file :ensure name specs))

(defn directory/ensure [name & specs]
  "Given a directory name and specification, return a directory ensure struct"
  (generic-resource :directory :ensure name specs))

(defn directory/remove [name & specs]
  "Given a directory name and specification, return a directory remove struct"
  (generic-resource :directory :remove name specs))

(defmacro host [host-name & host-definition]
  "The top-level wrapper used to define a host to be configured"
  ~(defn machine-config
     []
     {:metadata {:name ,host-name}
      :resources (group-by-action-and-type (flatten (tuple ,;host-definition)))}))

(defn group-by-action-and-type [data]
  "Turns an array of resources into a struct of structs"
  (defn collate [aggr item]
    (each resource-type (keys item)
      (let [resource (get item resource-type)
            action (get resource :action)]
        (when action
          (unless (get aggr action)
            (put aggr action @{}))
          (put aggr action
               (let [curr-map (get aggr action)
                     curr-list (get curr-map resource-type @[])]
                 (put curr-map resource-type (array/concat curr-list [resource]))
                 curr-map)))))
    aggr)

  (var ret (reduce collate @{} data))
  (each k (keys ret) (put ret k (table/to-struct (get ret k))))
  (table/to-struct ret))


(defn _group-by-action-and-type [data]
  "Turns an array of resources into a struct which holds two tables kese"
  (defn collate [aggr item]
    "Private function for reduce"
    (each resource-type (keys item)
      (let [resource (get item resource-type)
            action (get resource :action)]
        (when action
          (unless (get aggr action)
            (put aggr action @{}))
          (put aggr action
               (let [curr-map (get aggr action)
                     curr-list (get curr-map resource-type @[])]
                 (put curr-map resource-type (array/concat curr-list [resource]))
                 curr-map)))))
    aggr)
  (table/to-struct (reduce collate @{} data)))

(defn collect-resources
  [& resource-structs]
  (array ;resource-structs))

(defmacro role [role-name & role-definition]
  ~(defn ,role-name
     []
     (setdyn :role-dyn (string ',role-name))
     (collect-resources ,;role-definition)))
