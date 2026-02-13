(use ./lib)
(import ../collector)

(def doer :ipfilter)
(def description "Set or remove ipfilter rules.")
(def name-is "Any convenient name: not used internally")
(def mandatory-props-ensure
  {:priority {:types [:number]
              :help "rule resources are ordered by priority, lowest number first"}
   :always-reload {:types [:boolean]
                   :help "if any ipfilter/ensure resource sets this to true, then the
                    firewall rules will be reloaded every time Gurp runs,
                    regardless of whether the aggregated ipf.conf file has changed"}})
(def optional-props-ensure
  {:from {:types [:string]
          :help "Apply rules in the given file. If relative, looks in ../files"}
   :content {:types [:string]
             :help "Apply these rules. Must have :content xor :from"}})
(def mandatory-props-remove {})
(def optional-props-remove {})
(def defaults-ensure {:always-reload false})
(def defaults-remove {})

(defn ensure
  "Given rules or a path to a rules file, put an ensure struct in the collector"
  [name & spec]
  (if-not (has-exactly-one-of? [:content :from] spec)
    (error "need exactly one of :content or :from"))

  (collector/push :ensure doer (make-ensure-resource)))

(defn remove
  "Put a remove struct in the collector"
  [name & spec]
  (collector/push :remove doer (make-remove-resource)))

(def notes
  ["We build a single big set of filter rules from multiple sources, check its
    validity, and ensure its contents align with those of `/etc/ipf/ipf.conf`.
    If the file has changed, or if any resource used to build the content has
    `:always-reloaded true`, the contents of the file become the current
    firewall configuration."
   "The doer automatically enables the ipfilter service."
   "We do not (currently) support any additional `ipf` options."
   "Per-zone rules are not supported."
   "ipfilter/remove removes ALL filter rules"])
