(use judge)
(use ../lib/gurp)

(deftest publisher
  (test
    (publisher/ensure "sysdef"
                      :uri "http://pkg.lan.id264.net")
    {:publisher {:_id "/NO-ROLE/publisher/sysdef"
                 :action :ensure
                 :name "sysdef"
                 :uri "http://pkg.lan.id264.net"}})

  (test-error
    (publisher/ensure "sysdef")
    "publisher missing required key(s): uri")

  (test
    (publisher/remove "sysdef")
    {:publisher {:_id "/NO-ROLE/publisher/sysdef"
                 :action :remove
                 :name "sysdef"}}))
