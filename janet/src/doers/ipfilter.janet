(use ./lib)
(import ../collector)

(def doer :ipfilter)
(def description "Set or remove ipfilter rules.")
(def name-is "Any convenient name: not used internally")
(def mandatory-props-ensure
  {:priority {:types [:number]
              :help "rule resources are ordered by priority, lowest number first"}})
(def optional-props-ensure
  {:from {:types [:string]
          :help "Apply rules in the given file. If relative, looks in ../files"}
   :content {:types [:string]
             :help "Apply these rules. Must have :content xor :from"}})
(def mandatory-props-remove {})
(def optional-props-remove {})
(def defaults-ensure {})
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
