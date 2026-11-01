(use ./lib/definers)
(import ../collector)

(def match-allowed ["exact" "starts-with" "ends-with" "contains" "regex"])
(def apply-to-allowed ["all" "first" "last"])

(defdoer :file-line
  "Ensure lines do or do not exist in the given file."
  :name-is "Fully qualified path to file"

  (ensure
    (property "insert-at" [:number] :optional
              "If a new line must be added, it will go at this index")
    (property "line" [:string] :optional "The line which must exist")
    (property "replace" [:string] :optional "Pattern to replace. Rust regex")
    (property "with" [:string] :optional "Counterpart to :replace")
    (property "apply-to" [:string] :optional
              "Which matches to act on when replacing"
              :values apply-to-allowed))

  (remove
    (property "pattern" [:string] :mandatory
              "The line or pattern which must be removed")
    (property "match" [:string] :mandatory "How to match the line"
              :values match-allowed
              :default (first match-allowed))
    (property "apply-to" [:string] :mandatory "Which matches to act on"
              :values apply-to-allowed
              :default (first apply-to-allowed)))

  (notes
    "The file is not managed here. Use a file resource."
    "The doer reads the whole file into memory, so be mindful of file size."
    "Appended lines have a newline at the beginning and end."
    "Removing a line puts a newline on the end of the file if there wasn't one
     already."
    "Files are not backed up."))

(make-fn :ensure :file-line)
(make-fn :remove :file-line)
