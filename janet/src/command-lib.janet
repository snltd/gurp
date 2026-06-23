(defn strip-ansi
  "Remove ANSI control codes from the given string"
  [str]
  (peg/replace-all
    '(* "\e[" (any (set "0123456789;")) (range "az" "AZ"))
    ""
    str))
