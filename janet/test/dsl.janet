(use judge)
(use ../src/dsl)

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
  (test (zfscat big-pool "") "big"))

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

# (deftest run-cmd
#   (test (run-cmd "echo hello") "hello")
#   (test (run-cmd "ls -d /usr") "/usr")
#   (test-error
#     (run-cmd "/no/such/thing --verbose")
#     "@[\"/no/such/thing\" \"--verbose\"]: No such file or directory"))

(deftest parent
  (test (parent "/") "/")
  (test (parent "/path/to/file") "/path/to"))

(deftest cron-minutes-from-name
  (test (cron-minutes-from-name "tester" 15) "3,18,33,48")
  (test (cron-minutes-from-name "tester" 10) "3,13,23,33,43,53")
  (test (cron-minutes-from-name "tester" 30) "3,33")
  (test (cron-minutes-from-name "test-host-2" 30) "14,44")
  (test (cron-minutes-from-name "test-host-2" 20) "14,34,54"))

(deftest repeated-line-file
  (test
    (repeated-line-file "%d: this is the %s line" [[1 :first] [2 :second] [3 :third]])
    "1: this is the first line\n2: this is the second line\n3: this is the third line\n")

  (test
    (repeated-line-file "this is the %s line" [:first :second :third])
    "this is the first line\nthis is the second line\nthis is the third line\n"))

(deftest indoc
  (test
    (indoc "flat line") "flat line\n")

  (test
    (indoc ```
    line1
    line2
    
    ```)
    "line1\nline2\n\n")

  (test
    (indoc `
      first line indented
    others
    not`)
    "  first line indented\nothers\nnot\n")

  (test
    (indoc `
      gibbus
         and
      chubb`)
    "gibbus\n   and\nchubb\n")

  (test-error
    (indoc 123)
    "indoc: expected a string literal, got 123"))

(deftest template
  (test
    (template-out
      "I, {{ sentiment}}, {{ sentiment }} {{ language }}"
      {:sentiment "like" :language "Janet"})
    "I, like, like Janet")

  (test
    (template-out
      "I {{sentiment    }} {{ language}} too"
      {:sentiment "like" :language "Rust"})
    "I like Rust too")

  (test-error
    (template-out
      "I {{ sentiment }} {{ language }} though"
      {:sentiment "don't much care for" :oops "things like" :language "YAML"})
    "unused vars: expected sentiment, language: got sentiment, language, oops")

  (test-error
    (template-out
      "I also {{ sentiment }} {{ verb }} {{ amount }} of {{ language }}"
      {:sentiment "enjoy" :language "Ruby"})
    "unpopulated fields in template: {{ verb }}, {{ amount }}"))

(deftest qualified-path?
  (test (qualified-path? "/this/is/qualified") true)
  (test (qualified-path? "and/this/is/not") false))

(deftest qualify-from-path-without-dyn
  (test (qualify-from-path "/this/is/qualified") "/this/is/qualified")
  (test-error
    (qualify-from-path "and/this/is/not")
    "cannot qualify path for and/this/is/not: gurp-config-root is not set"))

(deftest qualify-from-path-with-dyn
  (setdyn :gurp-config-root "/test/root")
  (test (qualify-from-path "/this/is/qualified") "/this/is/qualified")
  (test
    (qualify-from-path "some/path")
    "/test/root/files/some/path"))
