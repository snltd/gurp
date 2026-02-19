(use judge)
(import ../../../src/doers/zone)

(deftest zone/fs
  (test
    (zone/fs "/home" :special "/export/home")
    {:fs @{:dir "/home"
           :special "/export/home"
           :type "lofs"}})
  (test
    (zone/fs "/data" :special "/export/data")
    {:fs @{:dir "/data"
           :special "/export/data"
           :type "lofs"}})

  (test-error
    (zone/fs "/home"
             :special "/export/home"
             :oops "wat?")
    "In zone/fs /home: unexpected property :oops. Valid properties are :dir, :special, :type, :options, :label")

  (test-error
    (zone/fs "/dir")
    "In zone/fs /dir: did not find mandatory property :special. Mandatory properties are :dir, :special"))
