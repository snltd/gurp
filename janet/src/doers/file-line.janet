(use ./lib)
(import ../collector)

(def doer :file-line)
(def description "Ensure lines do or do not exist in the given file.")
(def name-is "Fully qualified path to file")

(def match-allowed ["exact" "starts_with" "ends_with" "contains" "matches"])
(def apply-to-allowed ["all" "first" "last"])

(def mandatory-ensure-props {})
(def optional-ensure-props
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
(def mandatory-remove-props
  {:pattern {:types [:string]
             :help "The line or pattern which must be removed"}})
(def optional-remove-props
  {:match {:types [:string]
           :help (string "How to match the line: " (comma-sep match-allowed))}
   :apply-to {:types [:string]
              :help (string "Which matches to act on: "
                            (comma-sep apply-to-allowed))}})
(def default-ensure-prop-values {})
(def default-remove-prop-values {:match "exact"
                                 :apply-to "all"})

(defn ensure
  "Given a path and specification, put an ensure struct in the collector"
  [name & spec]
  (collector/push :ensure doer (make-ensure-resource)))

(defn remove
  "Given a path and specification, put a remove struct in the collector"
  [name & spec]
  (def spec-struct (make-spec-struct spec))

  (if-let [match-val (spec-struct :match)]
    (if-not (has-value? match-allowed match-val)
      (error
        (string "match must be one of "
                (comma-sep match-allowed) " [Got '" match-val "']"))))

  (if-let [type-val (spec-struct :apply-to)]
    (if-not (has-value? apply-to-allowed type-val)
      (error
        (string "type must be one of " (comma-sep apply-to-allowed)))))

  (def all-specs (spec-with-defaults default-ensure-prop-values spec-struct))
  (def safe-specs (checked-spec all-specs
                                mandatory-ensure-props
                                optional-ensure-props))

  (collector/push :remove doer (spec->resource doer name safe-specs)))
