(use ./lib)
(import ../collector)

(def match-allowed ["exact" "starts-with" "ends-with" "contains" "regex"])
(def apply-to-allowed ["all" "first" "last"])

(defdoer :file-line
  "Ensure lines do or do not exist in the given file."
  :name-is "Fully qualified path to file"

  :optional-props-ensure
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
                            (comma-sep apply-to-allowed))}}

  :mandatory-props-remove
  {:pattern {:types [:string]
             :help "The line or pattern which must be removed"}
   :match {:types [:string]
           :help "How to match the line"
           :values match-allowed}
   :apply-to {:types [:string]
              :help "Which matches to act on"
              :values apply-to-allowed}}

  :defaults-remove
  {:match "exact"
   :apply-to "all"}

  :notes
  ["The file is not managed here. Use a file resource."
   "The doer reads the whole file into memory, so be mindful of file size."
   "Appended lines have a newline at the beginning and end."
   "Removing a line puts a newline on the end of the file if there wasn't one
    already."
   "Files are not backed up."])

(make-ensure-fn "file-line")
(make-remove-fn "file-line")
