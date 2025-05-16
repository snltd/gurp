# This will end up being embededd in the Rust executable. (With options
# to dump it to stdout and to replace it with a user-supplied version.)

(use ../defaults)

(def id-sep "/")

(defn flesh-out-resource
  "Adds the given 'default' proto to a single resource"
  [resource resource-type default-values]
  (def role (get resource :role))
  (put resource :role nil)
  # we probably won't need :name either
  (def resource-name (get resource :name))
  (put resource :_id (string/join [role resource-type resource-name] id-sep))
  (table/setproto resource default-values)
  (table/proto-flatten resource))

(defn flesh-out-resources
  "Adds a suitable 'default' proto to an array of resources, if one exists"
  [resource-type resource-list]
  (let [resource-proto (get defaults resource-type)]
    (if (nil? resource-proto)
      resource-list
      (map (fn [r] (flesh-out-resource r resource-type resource-proto)) resource-list))))

(defn merge-protos
  "Adds default values to all resources of all types"
  [resource-hash]
  (reduce
    (fn [acc [resource-type resource-list]]
      (put acc resource-type (flesh-out-resources resource-type resource-list))
      acc)
    (table)
    (pairs resource-hash)))

# The `host` macro defines a machine configuration for a host by composing
# roles.
#
# It expands into a `machine-config` function which aggregates and merges the
# resource definitions from all specified roles. Each role must be previously
# defined and resolve to a function which returns a role table.
#
# Arguments:
#   name  – A symbolic name for the host (used in metadata).
#   &body – A keyword argument list that must include `:roles`, an array of
#           role names (symbols or strings) to include in the host.
#
# Returns:
#   A function `machine-config` that returns a table containing:
#     :metadata – A table with the host's name.
#     :resources – A merged table of resources from all roles.
(defmacro host [host-name & host-definition]
  ~(defn machine-config
     []
     (def roles (array))
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
       (setdyn :host-role host-role)
       (array/push roles (helpers/merge-protos (fn-to-call))))
     (table :metadata @{:name ,host-name}
            :resources
            (apply helpers/merge-role-tables roles))))

(defmacro role [role-name & role-definition]
  "Turn a role definition into a table"
  ~(defn ,role-name
     []
     (table ,;role-definition)))

(defmacro ensure
  "Turn a resource definition into a table, taged with 'ensure'"
  [resource-name & resource-definition]
  ~(merge (table ,;resource-definition)
          {:name ,resource-name
           :role (dyn :role-name)
           :action "ensure"}))

(defmacro remove
  "Turn a resource definition into a table, tagged with 'remove'"
  [resource-name & resource-definition]
  ~(merge (table ,;resource-definition)
          {:name ,resource-name
           :role (dyn :role-name)
           :action "remove"}))

(defn merge-role-tables
  "Merges multiple role tables into the single flat array of resources"
  [& role-tables]
  (print "-----------------------")
  (pp role-tables)
  (print "-----------------------")
  (let [result (array)]
    (each role-table role-tables
      (each resource-type (keys role-table)
        # (var resources (get result resource-type @[]))
        (array/concat result 
             (get role-table resource-type))))
    result))
