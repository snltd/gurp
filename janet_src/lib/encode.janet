(defn- escape [s]
  (def num-bytes (length s))
  (def b (buffer/new num-bytes))
  (var i 0)
  (while (< i num-bytes)
    (def c (get s i))
    (buffer/push b (cond
                     (= c 0x08)
                     "\\b"
                     (= c 0x09)
                     "\\t"
                     (= c 0x0A)
                     "\\n"
                     (= c 0x0C)
                     "\\f"
                     (= c 0x0D)
                     "\\r"
                     (= c 0x22)
                     "\\\""
                     (= c 0x5C)
                     "\\\\"
                     (< c 0x20)
                     (string/format "\\u%04x" c)
                     # 1-byte variant (0xxxxxxx)
                     (< c 0x80)
                     c
                     # 2-byte variant (110xxxxx 10xxxxxx)
                     (< 0xBF c 0xE0)
                     (string/format "\\u%04x"
                                    (bor (blshift (band c 0x1F) 6)
                                         (band (get s (++ i)) 0x3F)))
                     # 3-byte variant (1110xxxx 10xxxxxx 10xxxxxx)
                     (< c 0xF0)
                     (string/format "\\u%04x"
                                    (bor (blshift (band c 0x0F) 12)
                                         (blshift (band (get s (++ i)) 0x3F) 6)
                                         (band (get s (++ i)) 0x3F)))
                     # 4-byte variant (11110xxx 10xxxxxx 10xxxxxx 10xxxxxx)
                     (< c 0xF8)
                     (do
                       (def cp (bor (blshift (band c 0x07) 18)
                                    (blshift (band (get s (++ i)) 0x3F) 12)
                                    (blshift (band (get s (++ i)) 0x3F) 6)
                                    (band (get s (++ i)) 0x3F)))
                       (def hi (+ (brshift (- cp 0x10000) 10) 0xd800))
                       (def lo (+ (band (- cp 0x10000) 0x3ff) 0xdc00))
                       (string/format "\\u%04x\\u%04x" hi lo))
                     (error (string "invalid byte:" c))))
    (++ i))
  b)


(defn encode
  "Encodes a Janet data structure into JSON. Pass :pretty? true for formatted output."
  [data &keys {:pretty? pretty?}]
  (default pretty? false)
  (var res @"")
  (var indent 0)

  (defn push-indent []
    (when pretty? (buffer/push res "\n" (string/repeat " " indent))))

  (defn encode-internal [value]
    (cond
      (nil? value)
      (buffer/push res "null")

      (boolean? value)
      (buffer/push res (if value "true" "false"))

      (number? value)
      (buffer/push res (describe value))

      (string? value)
      (buffer/push res "\"" (escape value) "\"")

      (bytes? value)
      (buffer/push res "\"" (escape value) "\"")

      (symbol? value)
      (buffer/push res "\"" (escape (string value)) "\"")

      (indexed? value)
      (do
        (buffer/push res "[")
        (when pretty? (+= indent 2))
        (var first? true)
        (each item value
          (unless first?
            (buffer/push res ",")
            (push-indent))
          (when first?
            (when pretty? (push-indent)))
          (encode-internal item)
          (set first? false))
        (when pretty? (+= indent -2) (push-indent))
        (buffer/push res "]"))

      (dictionary? value)
      (do
        (buffer/push res "{")
        (when pretty? (+= indent 2))
        (var first? true)
        (eachp [k v] value
          (unless first?
            (buffer/push res ",")
            (push-indent))
          (when first?
            (when pretty? (push-indent)))
          (buffer/push res "\"" (escape k) "\":")
          (when pretty? (buffer/push res " "))
          (encode-internal v)
          (set first? false))
        (when pretty? (+= indent -2) (push-indent))
        (buffer/push res "}"))

      (error (string "cannot encode " (type value) " to JSON: " value))))

  (encode-internal data)
  res)
