#!/usr/bin/env janet
#
# Generate Markdown documentation for all the doers, using the definition file
# and code examples which are also uses in tests.
# 
(use ../src/doers)
(use ../src/commands)

(defn main
  [_cmd & args]
  (loop [arg :in args]
    (print (markdown-for-doer arg))))
