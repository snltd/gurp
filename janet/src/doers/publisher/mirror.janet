(use ../lib)

(defhelper :publisher :mirror
  "Define a mirror when managing a publisher."
  :name-is "The mirror URI"

  :mandatory-props
  {:name {:types [:string]
          :help "URI of mirror"}}

  :optional-props
  {:proxy {:types [:string]
           :help "Proxy URI for this mirror"}})

(defn mirror
  "Given a spec, return a publisher mirror struct."
  [name & spec]
  (let [spec-struct (make-spec-struct :name name ;spec)
        spec-table (pinpoint-error :mirror
                                   (checked-spec spec-struct
                                                 mandatory-props-mirror
                                                 optional-props-mirror))]

    (struct :mirror spec-table)))
