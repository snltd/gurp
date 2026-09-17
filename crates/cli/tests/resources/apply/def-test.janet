# Used in functional tests. There is a corresponding .jimage file. You should
# be able to regenerate it from this file with
#
# $ gurp compile --format=jimage --output-file=def-test.jimage def-test.janet
#
# If you can't, you've found a bug.
# 
(host "test"
      (if (user-def :test-def)
        (print "found a definition")
        (print "no definition found"))

      (print (string/format "gurp-config-root is %p" (dyn :gurp-config-root))))
