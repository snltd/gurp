(use ./lib)
(import ../collector)

(def doer :file-line)
(def description "Ensure lines do or do not exist in the given file.")
(def name-is "Fully qualified path to file")

(def match-allowed ["exact" "starts-with" "ends-with" "contains" "regex"])
(def apply-to-allowed ["all" "first" "last"])

(def mandatory-props-ensure {})
(def optional-props-ensure
  {:insert-at {:types [:number]
               :help "If a new line must be added, it will go at this index"}
   :line {:types [:string]
          :help "The line which must exist"}
   :replace {:types [:string]
             :help "Pattern to replace. Rust regex"}
   :with {:types [:string]
          :help "Counterpart to :replace"}
   :apply-to {:types [:string]
              :help (string "Which matches to act on when replacing: "
                            (comma-sep apply-to-allowed))}})
(def mandatory-props-remove
  {:pattern {:types [:string]
             :help "The line or pattern which must be removed"}
   :match {:types [:string]
           :help (string "How to match the line: " (comma-sep match-allowed))}
   :apply-to {:types [:string]
              :help (string "Which matches to act on: "
                            (comma-sep apply-to-allowed))}})
(def optional-props-remove {})
(def defaults-ensure {})
(def defaults-remove {:match "exact"
                      :apply-to "all"})

(defn ensure
  "Given a path and specification, put an ensure struct in the collector"
  [name & spec]
  (collector/push :ensure doer (make-ensure-resource)))

(defn remove
  "Given a path and specification, put a remove struct in the collector"
  [name & spec]
  (def spec-struct (make-spec-struct ;spec))

  (pinpoint-error
    "remove"
    (if-let [match-val (spec-struct :match)]
      (if-not (has-value? match-allowed match-val)
        (errorf "match must be one of %s [Got '%s']"
                (comma-sep match-allowed)
                match-val)))

    (if-let [type-val (spec-struct :apply-to)]
      (if-not (has-value? apply-to-allowed type-val)
        (errorf "type must be one of " (comma-sep apply-to-allowed)))))

  (def all-specs (spec-with-defaults defaults-remove spec-struct))
  (def safe-specs (checked-spec all-specs
                                mandatory-props-remove
                                optional-props-remove))

  (collector/push :remove doer (spec->resource doer name safe-specs)))

(def notes
  ["The file is not managed here. Use a file resource."
   "The doer reads the whole file into memory, so be mindful of file size."
   "Appended lines have a newline at the beginning and end."
   "Removing a line puts a newline on the end of the file if there wasn't one
    already."
   "Files are not backed up."])
