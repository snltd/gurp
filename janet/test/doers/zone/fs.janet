(use judge)
(import ../../../src/doers/zone)

(deftest zone/fs
  (test
    (zone/fs "/home" :special "/export/home")
    [:fs
     @{:from-helper true
       :name "/home"
       :special "/export/home"
       :type "lofs"
       :unique false}])
  (test
    (zone/fs "/data" :special "/export/data")
    [:fs
     @{:from-helper true
       :name "/data"
       :special "/export/data"
       :type "lofs"
       :unique false}])

  (test-error
    (zone/fs "/home"
             :special "/export/home"
             :oops "wat?")
    "In zone/fs /home: unexpected property :oops. Valid properties are :label, :options, :special, :type")

  (test-error
    (zone/fs "/dir")
    "In zone/fs /dir: did not find mandatory property :special. Mandatory properties are :special, :type"))
