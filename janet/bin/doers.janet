#!/usr/bin/env janet
#
# List the doers like 'gurp doers'. Pass it any argument and it won't bold the
# doer names.
# 
(use ../src/doers)
(use ../src/commands)
(use ../src/command-helpers)

(defn main [_cmd & args]
  (print
    ((comp (if (= (first args) "-C") strip-ansi identity)
           list-doers))))
