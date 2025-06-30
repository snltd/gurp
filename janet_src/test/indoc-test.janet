(use judge)
(use ../lib/gurp)

(test-macro
  (indoc merp `
      this file

       should be
      Left aligned`)
  (def merp (string (if-not (string? "      this file\n\n       should be\n      Left aligned") (error "indoc: expected a string literal")) (-> (->> "      this file\n\n       should be\n      Left aligned" (string/split "\n") (filter (short-fn (not (empty? (string/trim $))))) (map (short-fn (peg/find :S $))) (min-of) (string/repeat " ") (string "\n")) (string/split (string "\n" "      this file\n\n       should be\n      Left aligned")) (string/join "\n") (string/trim)))))

(deftest "indoc"
  (test
    (indoc tester `
      gibbus
         and
      chubb`)
    "gibbus\n   and\nchubb"))
