(defn title-words [& words]
  (defn title-word [word]
    (peg/replace 1 string/ascii-upper word))
  (string/join (map title-word words) " "))

(defn h1 [text]
  (string "# " text "\n"))

(defn h2 [text]
  (string "## " text "\n"))

(defn h3 [text]
  (string "### " text "\n"))

(defn code [text]
  (string "`" text "`"))

(defn table-header [& cols]
  (string
    "|  " (string/join cols "  |  ") "  |\n"
    "|--" (string/join (map |(string/repeat "-" (length $)) cols) "--|--") "--|\n"))

(defn table-row [& fields]
  (string "| " (string/join ;fields " | ") " |\n"))

(defn code-block
  [code]
  (string "```janet\n" (string/trim code) "\n```\n\n"))
