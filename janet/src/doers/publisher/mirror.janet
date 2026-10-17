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

(make-helper-fn "mirror")
