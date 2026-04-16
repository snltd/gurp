(use ./lib)
(use ../dsl)
(import ../collector)

(def doer :system-cert)
(def description "Add and remove system TLS certificates")
(def name-is "File certificate will have in /etc/ssl/certs")
(def optional-props-ensure
  {:from
   {:types [:string]
    :help "Copy cert content from this file. If relative, looks in ../files"}
   :content
   {:types [:string]
    :help "Use this literal string as the cert"}
   :from-url
   {:types [:string]
    :help "Fetch cert from the given URL"}})
(def mandatory-props-ensure {})
(def mandatory-props-remove {})
(def optional-props-remove {})
(def defaults-ensure {})
(def defaults-remove {})

(defn ensure
  "Given a cert or reference to one, put an ensure struct in the collector"
  [name & spec]
  (pinpoint-error
    :ensure
    (if-not (has-exactly-one-of? [:content :from :from-url] spec)
      (error "Provide exactly one of :content, :from, :from-url")))

  (def spec-table
    (expand-from-value (struct/to-table (make-spec-struct ;spec))))

  (def all-specs (spec-with-defaults defaults-ensure spec-table))

  (def safe-specs
    (pinpoint-error
      :ensure
      (checked-spec all-specs
                    mandatory-props-ensure
                    optional-props-ensure)))

  (collector/push :ensure doer (spec->resource doer name safe-specs)))

(defn remove
  "Given a cert name, put a remove struct in the collector"
  [name & spec]
  (collector/push :remove doer (make-remove-resource)))

(def notes
  ["Does not generate certs: just copies them to the system directory and
    re-hashes it."
   "If a `:from` path is relative, Gurp will fully qualify it using the same
    rules as the `file` doer."])
