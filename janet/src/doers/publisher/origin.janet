(use ../lib)

(defhelper :publisher :origin
  "Define an origin when managing a publisher."
  :name-is "The origin URI"

  :mandatory-props
  {:name {:types [:string]
          :help "URI of origin"}}

  :optional-props
  {:proxy {:types [:string]
           :help "Proxy URI for this origin"}})

(defn origin
  "Given a spec, return a publisher origin struct."
  [name & spec]
  (let [spec-struct (make-spec-struct :name name ;spec)
        spec-table (pinpoint-error :origin
                                   (checked-spec spec-struct
                                                 mandatory-props-origin
                                                 optional-props-origin))]

    (struct :origin spec-table)))
