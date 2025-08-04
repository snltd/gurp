(use judge)
(use ../lib/gurp)

(deftest "check-unique-ids"
  (test
    (check-unique-ids
      [{:_id "/test/file/a" :from "a" }
       {:_id "/test/file/b" :from "b"}
       {:_id "/test/file/c" :from "c"}
      ])
    true)

  (test-error
    (check-unique-ids
      [{:_id "/test/file/a" :from "a" }
       {:_id "/test/file/b" :from "b"}
       {:_id "/test/file/b" :from "c"}
      ])
    "duplicate key: /test/file/b")
)
