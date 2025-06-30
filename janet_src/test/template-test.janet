(use judge)
(use ../lib/gurp)

(deftest "templates"
  (test
    (template-out
      "I {{ sentiment }} {{ language }}"
      {:sentiment "like" :language "Janet"})
    "I like Janet")

  (test
    (template-out
      "I {{sentiment    }} {{ language}} too"
      {:sentiment "like" :language "Rust"})
    "I like Rust too")

  (test-error
    (template-out
      "I {{ sentiment }} {{ language }} though"
      {:sentiment "don't much care for" :oops "things like" :language "YAML"})
    "unused vars")

  (test-error
    (template-out
      "I also {{ sentiment }} {{ verb }} {{ amount }} of {{ language }}"
      {:sentiment "enjoy" :language "Ruby"})
    "unpopulated fields in template: {{ verb }}, {{ amount }}"))
