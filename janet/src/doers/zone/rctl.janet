(import ../../collector)
(use ../definers)

(defhelper :zone :rctl
  "Define a resource control when creating a zone."
  :name-is "RCTL name"
  (helper
    (property "priv" [:string] :mandatory "rctl privilege"
              :default "privileged")
    (property "action" [:string] :mandatory "rctl action"
              :default "deny")
    (property "name" [:string] :mandatory "private field managed by Gurp")
    (property "limit" [:number] :mandatory "rctl limit value")))

(make-helper-fn :rctl)
