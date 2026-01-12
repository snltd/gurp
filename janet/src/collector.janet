(defn new-collector
  "Returns an empty *collector*. In a function as most tests use it"
  [] @{:ensure @{} :remove @{}})

# Yes, a GLOBAL VARIABLE! It collects all the resources from the host we
# are configuring. :ensure and :remove are tables whose keys are resource
# types and values are arrays of resources
# 
(var *collector* (new-collector))

(defn push
  "Push a resource onto the appropriate array inside the collector."
  [action resource-type resource]
  (def action-struct (*collector* action))

  (if-not (has-key? action-struct resource-type)
    (set (action-struct resource-type) @[]))

  (def resource-array (action-struct resource-type))

  (array/concat resource-array resource))
