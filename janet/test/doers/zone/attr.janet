(use judge)
(import ../../../src/doers/zone)

(deftest zone/attr
  (test
    (zone/attr "turn-it-on" :value false)
    {:attr @{:name "turn-it-on"
             :type "boolean"
             :value false}})

  (test
    (zone/attr "spandau-ballet-number-1" :value true :type "astring")
    {:attr @{:name "spandau-ballet-number-1"
             :type "astring"
             :value "true"}})

  (test
    (zone/attr "this-is-a-number" :value 123)
    {:attr @{:name "this-is-a-number"
             :type "uint"
             :value 123}})

  (test
    (zone/attr "kernel-ver" :value "4.4")
    {:attr @{:name "kernel-ver"
             :type "string"
             :value "4.4"}})
  (test-error
    (zone/attr "thing" :type "astring")
    "did not find mandatory property :value. Mandatory properties are :name, :value"))
