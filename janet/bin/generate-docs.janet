#!/usr/bin/env janet
#
# Generate Markdown documentation for all the doers, using the definition file
# and code examples which are also uses in tests.
# 
(setdyn :import-libs-relative true)
(use ../src/doers)
(use ../src/commands)
(use ../src/build-docgen)

(setdyn :repo-root
  (->>
    (dyn *current-file*)
    (os/realpath)
    (peg/replace '(* "/janet" (some 1)) "")))

   
(defn main
  [_cmd & args]
  (if (= (first args) "ALL")
    (generate-all-docs)
    (generate-docs-to-stdout args)))
