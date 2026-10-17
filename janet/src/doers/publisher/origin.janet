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

(make-helper-fn "origin")
