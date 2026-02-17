(use ./lib)
(import ../collector)

(def doer :ipnat)
(def description "Set or remove NAT rules.")
(def name-is "Any convenient name: not used internally")
(def mandatory-props-ensure
  {:priority {:types [:number]
              :help "NAT rule resources are ordered by priority, lowest number first"}})
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
    (pinpoint-error
      :ensure
      (error "need exactly one of :content or :from")))

  (collector/push :ensure doer (make-ensure-resource)))

(defn remove
  "Put a remove struct in the collector"
  [name & spec]
  (collector/push :remove doer (make-remove-resource)))

(def notes
  ["Gurp assembles bundles of NAT rules, given as literal strings (`:content`)
    or in files (`:from`). Ordering comes from the `:priority` value, lowest
    first. When all rules have been assembled, the list is verified and compared
    against the currently loaded NAT rules. If different, the new rules are
    applied and written to `/etc/ipf/ipnat.conf`."
   "Every run asserts the live and persistent state of the NAT table."
   "No ipnat flags (-R, -r etc) are supported."
   "It's too tricky to support local-zone-from-global-zone NAT rules, so we
    don't."
   "The doer automatically enables the ipfilter service."
   "ipnat/remove removes ALL NAT rules"])
