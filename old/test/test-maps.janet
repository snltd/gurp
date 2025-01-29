(use judge)
(use ../lib/maps)

(deftest extract-id-and-val-fields
  (test (extract-id-and-val-fields ["123" "456" "abc" "789" "def"] 2 3) [nil "789"])
  (test (extract-id-and-val-fields ["123" "456" "abc" "789" "def"] 2 3) [nil "789"])
  (test-error (extract-id-and-val-fields ["123" "456" "abc" "789" "def"] 20 1) "bad slot #0, expected string, symbol, keyword or buffer, got nil"))

(deftest generate-maps
  (test (generate-passwd-map "test/resources/passwd")
    {0 "root"
     1 "daemon"
     2 "bin"
     3 "sys"
     4 "adm"})
  (test (generate-group-map "test/resources/group")
    {0 "root"
     1 "other"
     2 "bin"
     3 "sys"
     4 "adm"
     5 "uucp"
     6 "mail"
     14 "sysadmin"}))

(deftest uid-maps
  (test ((uid->name "test/resources/passwd") 0) "root")
  (test ((uid->name "test/resources/passwd") 100) nil)
  (test ((name->uid "test/resources/passwd") "bin") 2)
  (test ((name->uid "test/resources/passwd") "whoever") nil))

(deftest group-maps
  (test ((gid->name "test/resources/group") 4) "adm")
  (test ((gid->name "test/resources/group") 14) "sysadmin")
  (test ((name->gid "test/resources/group") "sysadmin") 14))
