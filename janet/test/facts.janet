(use ../src/facts)
(use judge)

(deftest cache-hit
  (def test-hostname "dummy-test-hostname")

  # Should not hit the cache because it is empty
  (test (= (fact :hostname) test-hostname) false)

  (set (*fact-cache* :hostname) test-hostname)

  # Should hit the cache, so get a different value
  (test (= (fact :hostname) test-hostname) true))

(deftest uname
  (if (= (os/which) :illumos)
    (test (type (fact :uname)) :struct)))

(deftest unknown-fact
  (test-error (fact "wat") "unknown fact: wat"))
