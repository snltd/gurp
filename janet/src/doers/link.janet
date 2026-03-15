(use ./lib)
(import ../collector)

(def doer :link)
(def description "Create and remove links.")
(def name-is "Qualified path to the link that will be created")
(def mandatory-props-ensure
  {:source
   {:types [:string]
    :help "The file to which we will link"}
   :force-link
   {:types [:boolean]
    :help "If the link target already exists and this flag is true, Gurp will
           remove it and replace it with a link. If false, a pre-existing target
           causes an error"}
   :type
   {:types [:string]
    :help "The type of link: symbolic or hard"}})
(def optional-props-ensure {})
(def mandatory-props-remove {})
(def optional-props-remove {})
(def defaults-ensure
  {:type "symbolic"
   :force-link false})
(def defaults-remove {})

(defn ensure
  "Given target and source paths, put an ensure struct in the collector"
  [name & spec]
  (collector/push :ensure doer (make-ensure-resource)))

(defn remove
  "Given target and source paths, put a remove struct in the collector"
  [name & spec]
  (collector/push :remove doer (make-remove-resource)))

(def notes
  ["If the source doesn't exist, you get an error."
   "Files and directories are ensured before links, so you can link Gurp-managed
    resources."])
