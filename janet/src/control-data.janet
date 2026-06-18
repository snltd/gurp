# Control data controls how Gurp runs.

(import ./doers/lib :only [comma-sep] :prefix "")

(def control-keys
  "Permissible control keys"
  {:splay-seconds {:types [:number]
                   :help "Tells Gurp to pause by a random number of seconds up
                          to the given maximum before applying"}
   :gem-path {:types [:string]
              :help "Specify the gem binary used by the gem doer"}
   :metrics-to {:types [:string]
                :help "Send OpenTelemetry metrics to the given endpoint"}
   :logs-to {:types [:string]
             :help "Send OpenTelemetry logs to the given endpoint"}})

(defn new-control-data
  "Returns an empty *control-data*."
  []
  @{})

(var *control-data* (new-control-data))

(defn control-data
  "Adds to the *control-data* struct"
  [key value]
  (if-let [valid-key (control-keys key)]
    (if (has-value? (valid-key :types) (type value))
      (if-let [current-value (get *control-data* key)]
        (errorf "control-data '%v' is already set to  %v" key current-value)
        (set (*control-data* key) value))
      (errorf "incorrect type for control-data key %v. Got %v expected %s"
              key
              (type value)
              (comma-sep (valid-key :types))))
    (errorf "unknown control-data key: %p. Keys are %s"
            key
            (comma-sep (keys control-keys)))))
