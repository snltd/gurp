#!/usr/bin/env janet
#
# Use a doer's definition to generate documentation.
#
# Useful for testing and developing docs. Not used in the build or execution
# of Gurp.
#
# Pass "-C" to not have ANSI colouring.
# 
(use ../src/doers)
(use ../src/commands)
(use ../src/command-helpers)
(use ../src/doer-docs/describe-docs)

(defn main
  [_cmd & args]
  (print
    ((comp (if (= (first args) "-C") strip-ansi identity)
           help-for) (last args))))
