(use ./lib)
(import ../collector)

(defdoer :ip-properties
  "Sets global IP properties, via 'ipadm set-prop'."
  :name-is "Any convenient name: not used internally"
  :optional-props-ensure protocol-opts

  :notes
  ["Define `extra_priv_ports` as a comma-separated list."])

(defn ensure
  [name & spec]
  (->> ;spec
       (group-ip-properties mandatory-props-ensure optional-props-ensure)
       (pinpoint-error :ensure)
       (spec->resource doer name)
       (collector/push :ensure doer)))
