(use judge)
(use ../lib/gurp)

(deftest labelise
  (test (labelise "/some/file" 1 2 3) "_some_file-1-2-3")
  (test (labelise :key1 :key2 :key3) "key1-key2-key3")
  (test (labelise "string" 123 :keyword) "string-123-keyword"))
  

(deftest this
  (setdyn :role-dyn (string (quote basenode)))
  (test (this "file" "the-label" "owner") :/basenode/file/the-label/owner)
  (test (this "file" "the-label") :/basenode/file/the-label)
  (setdyn :role-dyn nil))

(deftest pathcat
  (def var1 "/opt/site")
  (def var2 "lib")
  (test (pathcat var1 "/chunk-a" var2 "chunk-b" "file.tar")
        "/opt/site/chunk-a/lib/chunk-b/file.tar")
  (test (pathcat "/opt/site/chunk-a/lib/chunk-b/file.tar")
        "/opt/site/chunk-a/lib/chunk-b/file.tar"))

(deftest zfscat
  (def big-pool "big")
  (test (zfscat big-pool "export" "flac") "big/export/flac")
  (test (zfscat big-pool "") "big")
  )

(deftest argcat
  (test (argcat "/bin/cat" "file1" "file2") "/bin/cat file1 file2")
  (test (argcat "judge" "test.janet") "judge test.janet"))

(deftest fields
  (test
    (fields "f1 f2 f3 f4    f5 ") @["f1" "f2" "f3" "f4" "f5"])
  (test
    (fields "     f1     f2
     f3 f4    f5 ")
    @["f1" "f2" "f3" "f4" "f5"]))

(deftest run-cmd
  (test (run-cmd "echo hello") "hello")
  (test (run-cmd "ls -d /usr") "/usr")
  (test-error
    (run-cmd "/no/such/thing --verbose")
    "@[\"/no/such/thing\" \"--verbose\"]: No such file or directory"))

# Inconsistent across development/test platforms
# (deftest hostname
#   (test (type (hostname)) :string))

(deftest parent
  (test (parent "/") "/")
  (test (parent "/path/to/file") "/path/to"))
