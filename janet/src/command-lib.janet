# Remove ANSI control codes from the given string
(defn strip-ansi [str]
  (peg/replace-all '(* "\e[" (any (set "0123456789;")) (range "az" "AZ"))
                   ""
                   str))
