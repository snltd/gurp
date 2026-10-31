(import ../../collector)
(use ../definers)

(defhelper :publisher :mirror
  "Define a mirror when managing a publisher."
  :name-is "The mirror URI"
  (helper
    (property "name" [:string] :mandatory "URI of mirror")
    (property "proxy" [:string] :optional "Proxy URI for this mirror")))

(make-helper-fn :mirror)
