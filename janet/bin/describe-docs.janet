#!/usr/bin/env janet
#
# Generate Markdown documentation for all the doers, using the definition file
# and code examples which are also uses in tests.
#
# Useful for testing and developing docs. Not used in the build or execution
# of Gurp.
# 
(use ../src/doers)
(use ../src/doer-docs/describe-docs)

(defn main
  [_cmd & args]
  (loop [doer :in args]
    (print (help-for-doer doer))))
