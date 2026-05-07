(use ../lib)

(def doer :publisher)
(def description-mirror "Define n mirror when managing a publisher.")
(def name-is-mirror "The mirror URI")
(def mandatory-props-mirror
  {:name {:types [:string]
         :help "URI of mirror"}})
(def optional-props-mirror
  {:proxy {:types [:string]
           :help "Proxy URI for this mirror"}})
(def defaults-mirror {})

(defn mirror
  "Given a spec, return a publisher mirror struct."
  [name & spec]
  (def spec-struct (make-spec-struct :name name ;spec))
  (def expanded-spec (spec-with-defaults defaults-mirror spec-struct))
  (def spec-table
    (pinpoint-error
      :mirror
      (checked-spec expanded-spec mandatory-props-mirror optional-props-mirror)))
  (struct :mirror spec-table))
