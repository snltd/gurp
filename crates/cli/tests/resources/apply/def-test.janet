(host "test"
  (if (user-def :test-def)
    (print "found a definition")
    (print "no definition found")))
