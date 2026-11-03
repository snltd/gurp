# A very basic and incomplete Markdown DSL

(import ./lib :only [join-words])

(defn h1 [& words]
  (string "# " (lib/join-words ;words) "\n"))

(defn h2 [& words]
  (string "## " (lib/join-words ;words) "\n"))

(defn h3 [& words]
  (string "### " (lib/join-words ;words) "\n"))

(defn code [& words]
  (string "`" (lib/join-words ;words) "`"))

(defn table-header [& cols]
  (string "|  " (string/join cols "  |  ") "  |\n"
          "|--"
          (string/join (map |(string/repeat "-" (length $)) cols) "--|--")
          "--|"))

(defn table-row [& fields]
  (string "| " (string/join fields " | ") " |"))

(defn code-block [code]
  (string "```janet\n" (string/trim code) "\n```\n"))
