(import ./references)

(defn new-collector
  "Returns an empty *collector*. In a function as most tests use it"
  [] @{:ensure @{} :remove @{}})

# Yes, a GLOBAL VARIABLE! It collects all the resources from the host we
# are configuring. :ensure and :remove are tables whose keys are resource
# types and values are arrays of resources
# 
(var *collector* (new-collector))

(defn push
  "Mutates global state by pushing a resource onto the appropriate array inside
  the collector. Returns the resource itself, which can be useful when 
  debugging with the REPL."
  [action resource-type resource]
  (def action-struct (*collector* action))

  (if-not (has-key? action-struct resource-type)
    (set (action-struct resource-type) @[]))

  (def resource-array (action-struct resource-type))
  (array/concat resource-array resource)
  resource)

(defn check-unique-ids
  "If the given list contains any duplicate resource IDs, throw an error"
  [resource-list]
  (var seen @{})
  (loop [id :in (map |($ :_id) resource-list)]
    (if (has-key? seen id)
      (errorf "duplicate key: %s" id)
      (set (seen id) true))))

(defn- finalise-action
  "When given a list of resources to ensure or remove, resolve any references,
  check there are no duplicate IDs, and return a fresh resource table"
  [resource-struct]
  (def all-resources (mapcat values resource-struct))
  (var ret @{})

  (loop [[resource-type resource-list] :pairs resource-struct]
    (do
      (check-unique-ids resource-list)
      (def resolved-resource-list (references/resolve resource-list all-resources))
      (set (ret resource-type) resolved-resource-list)))

  ret)

(defn finalise
  "Returns the resource struct that will be parsed by Serde in the Rust backend"
  [collector]
  (if (dyn :destroy-everything-you-touch)
    {:ensure {}
     :remove (finalise-action (collector :ensure))}
    {:ensure (finalise-action (collector :ensure))
     :remove (finalise-action (collector :remove))}))
