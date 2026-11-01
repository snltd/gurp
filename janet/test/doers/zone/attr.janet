(use judge)
(import ../../../src/doers/zone)

(deftest zone/attr
  (test
    (zone/attr "turn-it-on" :value false :type "boolean")
    [:attr
     @{:from-helper true
       :name "turn-it-on"
       :type "boolean"
       :unique false
       :value false}])

  (test
    (zone/attr "spandau-ballet-number-1" :value true :type "astring")
    [:attr
     @{:from-helper true
       :name "spandau-ballet-number-1"
       :type "astring"
       :unique false
       :value true}])

  (test
    (zone/attr "this-is-a-number" :value 123 :type "uint")
    [:attr
     @{:from-helper true
       :name "this-is-a-number"
       :type "uint"
       :unique false
       :value 123}])

  (test
    (zone/attr "kernel-ver" :value "4.4" :type "string")
    [:attr
     @{:from-helper true
       :name "kernel-ver"
       :type "string"
       :unique false
       :value "4.4"}])

  (test
    (zone/attr "extra" :value "-accel tcg,thread=multi" :type "string")
    [:attr
     @{:from-helper true
       :name "extra"
       :type "string"
       :unique false
       :value "-accel tcg,thread=multi"}]))

(deftest zone-attr-error
  (test-error
    (zone/attr "huh?")
    "In zone/attr huh?: did not find mandatory property :value. Mandatory properties are :type, :value")
        
  (test-error
    (zone/attr "thing" :value 123 :oops "wat")
    "In zone/attr thing: did not find mandatory property :type. Mandatory properties are :type, :value")

  (test-error
    (zone/attr "thing" :type "astring")
    "In zone/attr thing: did not find mandatory property :value. Mandatory properties are :type, :value"))
  
