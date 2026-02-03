#!/usr/bin/env janet
#
# List the doers like 'gurp doers'.
# 
(use ../src/doers)
(use ../src/commands)

(print (list-doers))
