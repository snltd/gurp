(def- default-protos
  {:file {:owner "root"
          :mode "0644"
          :group "root"}
   :cron {:hour "*"
          :minute "*"
          :day-of-month "*"
          :day-of-week "*"
          :month-of-year "*"
          :user "root"}
   :user {:shell "/bin/zsh"
          :primary-group "staff"}
   :directory {:owner "root"
               :mode "0755"
               :group "root"}})

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
      :_id (string "/" (dyn :role-dyn "NO-ROLE")
                   "/" resource-type
                   "/" (get (table ;resource-spec) :label
                            (string/replace-all "/" "_" resource-name)))
      :role (dyn :role-dyn)
      :name resource-name
      :action action
      (splice resource-spec))
    (struct/proto-flatten)
    (merge)
    (table/to-struct)
    (struct resource-type)))

(defn pkg/ensure [name &]
  "Given a a pkg name, return a pkg ensure struct. In OmniOS, the
  pkg version is effectively part of the name, so there are no parameters"
  (generic-resource :pkg :ensure name []))

(defn pkg/remove [name &]
  "Given a pkg name, return a pkg remove struct"
  (generic-resource :pkg :remove name []))

(defn file/ensure [name & specs]
  "Given a file name and specification, return a file ensure struct"
  (generic-resource :file :ensure name specs))

(defn file/remove [name & specs]
  "Given a file name and specification, return a file remove struct"
  (generic-resource :file :ensure name specs))

(defn file-line/ensure [name & specs]
  "Given a file name and a line pattern, make sure the file contains the line"
  (generic-resource :file-line :ensure name specs))

(defn file-line/remove [name & specs]
  "Given a file name and specification, make sure the file does not contain the line"
  (generic-resource :file-line :ensure name specs))

(defn directory/ensure [name & specs]
  "Given a directory name and specification, return a directory ensure struct"
  (generic-resource :directory :ensure name specs))

(defn directory/remove [name & specs]
  "Given a directory name and specification, return a directory remove struct"
  (generic-resource :directory :remove name specs))

(defn user/ensure [name & specs]
  "Given a user name and specification, return a user ensure struct"
  (generic-resource :user :ensure name specs))

(defn user/remove [name & specs]
  "Given a user name and specification, return a user remove struct"
  (generic-resource :user :remove name specs))

(defn cron/ensure [name & specs]
  "Given a name and specification, return a cron ensure struct"
  (generic-resource :name :ensure name specs))

(defn cron/remove [name & specs]
  "Given a name and specification, return a cron remove struct"
  (generic-resource :name :remove name specs))

(defn this-host
  "Returns the name of the host, set by a dyn in the host macro"
  []
  (dyn :host-dyn))

(defmacro host [host-name & host-definition]
  "The top-level wrapper used to define a host to be configured"
  (setdyn :host-dyn (string host-name))
  ~(defn machine-config
     []
     {:metadata {:name ,host-name}
      :resources (group-by-action-and-type (flatten (tuple ,;host-definition)))}))


(defn group-by-action-and-type [data]
  "Turns an array of resources into a struct of structs, and resolves references."

  (def flat-data (flatten (map values data)))

  (var resolve-reference nil)
  (set resolve-reference
       (fn [val seen]
         (if (keyword? val)
           (let [last-sep (last (string/find-all "/" val))
                 chunks (string/split "/" val last-sep)
                 id (first chunks)
                 field (keyword (last chunks))
                 referenced-struct (find (fn [a] (= id (get a :_id))) flat-data)]

             (if (nil? referenced-struct)
               (error (string/format "Referenced resource '%s' not found" id)))

             (if (has-value? seen id)
               (error (string/format "Detected circular reference [%q]" seen)))

             (array/push seen id)

             (def referenced-val (get referenced-struct field))

             (if (keyword? referenced-val)
               (resolve-reference referenced-val seen)
               referenced-val))
           val)))

  (defn resolve-resource [res]
    (var out @{})
    (each k (keys res)
      (put out k
           (if (= k :action) (get res k)
             (resolve-reference (get res k) @[]))))
    (table/to-struct out))

  (defn collate [aggr item]
    (each resource-type (keys item)
      (let [resource-raw (get item resource-type)
            resource (resolve-resource resource-raw)
            action (get resource :action)]
        (when action
          (unless (get aggr action)
            (put aggr action @{}))
          (put aggr action
               (let [curr-map (get aggr action)
                     curr-list (get curr-map resource-type @[])]
                 (put curr-map resource-type (array/concat curr-list [resource]))
                 curr-map))))) aggr)

  (var ret (reduce collate @{} data))
  (each k (keys ret) (put ret k (table/to-struct (get ret k))))
  (table/to-struct ret))


(defn _group-by-action-and-type [data]
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


(defn collect-resources
  [& resource-structs]
  (array ;resource-structs))

(defmacro role [role-name & role-definition]
  ~(defn ,role-name
     []
     (setdyn :role-dyn (string ',role-name))
     (collect-resources ,;role-definition)))
