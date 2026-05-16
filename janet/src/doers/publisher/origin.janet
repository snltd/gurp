(use ../lib)

(def doer :publisher)
(def description-origin "Define an origin when managing a publisher.")
(def name-is-origin "The origin URI")
(def mandatory-props-origin
  {:name {:types [:string]
         :help "URI of origin"}})
(def optional-props-origin
  {:proxy {:types [:string]
           :help "Proxy URI for this origin"}})
(def defaults-origin {})

(defn origin
  "Given a spec, return a publisher origin struct."
  [name & spec]
  (def spec-struct (make-spec-struct :name name ;spec))
  (def expanded-spec (spec-with-defaults defaults-origin spec-struct))
  (def spec-table
    (pinpoint-error
      :origin
      (checked-spec expanded-spec mandatory-props-origin optional-props-origin)))
  (struct :origin spec-table))
