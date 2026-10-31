(import ../../collector)
(use ../definers)

(defhelper :publisher :origin
  "Define an origin when managing a publisher."
  :name-is "The origin URI"
  (helper
    (property "name" [:string] :mandatory "URI of origin")
    (property "proxy" [:string] :optional "Proxy URI for this origin")))

(make-helper-fn :origin)
