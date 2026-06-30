(use judge)
(use ../src/gurp)

(deftest control-data
  (set *control-data* (new-control-data))
  (host "test"
        (control-data :gem-path "/opt/local/bin/gem")
        (control-data :splay-seconds 10))

  (test
    ((machine-config) :control-data)
    {:gem-path "/opt/local/bin/gem"
     :splay-seconds 10}))

(deftest control-data-unknown-key
  (set *control-data* (new-control-data))
  (test-error
    (control-data :wat "oops")
    "unknown control-data key: :wat. Keys are :splay-seconds, :metrics-to, :gem-path, :strict-hostname, :logs-to"))

(deftest control-data-wrong-type
  (set *control-data* (new-control-data))
  (test-error
    (control-data :splay-seconds "5")
    "incorrect type for control-data key :splay-seconds. Got :string expected :number"))

(deftest control-data-duplicate-key
  (set *control-data* (new-control-data))
    (control-data :splay-seconds 12)
  (test-error
    (control-data :splay-seconds 20)
    "control-data ':splay-seconds' is already set to  12"))
