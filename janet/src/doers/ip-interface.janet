(use ./lib)
(import ../collector)

(defdoer :ip-interface
  "Create and destroy IP interfaces, with optional properties. Properties are
  supplied with 'ip-interface-protocol'."

  :name-is "Interface name"
  :optional-props-ensure protocol-opts)

(defn ensure
  [name & spec]
  (->> ;spec
       (group-ip-properties mandatory-props-ensure optional-props-ensure)
       (pinpoint-error :ensure)
       (spec->resource doer name)
       (collector/push :ensure doer)))

(defremove "ip-interface")
