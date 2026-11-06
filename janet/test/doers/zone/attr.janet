(use judge)
(import ../../../src/doers/zone)

(deftest zone/attr
  (test
    (zone/attr "turn-it-on" :value false)
    [:attr @{:name "turn-it-on"
             :from-helper true
             :unique false
             :type "boolean"
             :value false}])

  (test
    (zone/attr "spandau-ballet-number-1" :value true :type "astring")
    [:attr @{:name "spandau-ballet-number-1"
             :from-helper true
             :unique false
             :type "astring"
             :value "true"}])

  (test
    (zone/attr "this-is-a-number" :value 123)
    [:attr @{:name "this-is-a-number"
             :from-helper true
             :unique false
             :type "uint"
             :value 123}])

  (test
    (zone/attr "kernel-ver" :value "4.4")
    [:attr @{:name "kernel-ver"
             :from-helper true
             :unique false
             :type "string"
             :value "\"4.4\""}])

  (test
    (zone/attr "extra" :value "-accel tcg,thread=multi")
    [:attr @{:name "extra"
             :from-helper true
             :unique false
             :type "string"
             :value "\"-accel tcg,thread=multi\""}])

  (test-error
    (zone/attr "huh?")
    "In zone/attr huh?: did not find mandatory property :value. Mandatory properties are :type, :value")

  (test-error
    (zone/attr "thing" :value 123 :oops "wat")
    "In zone/attr thing: unexpected property :oops. Valid properties are :label, :type, :value")

  (test-error
    (zone/attr "thing" :type "astring")
    "In zone/attr thing: did not find mandatory property :value. Mandatory properties are :type, :value"))
