(defn underline
  "Underline the given string"
  [text]
  (string "\x1b[4m" text "\x1b[0m"))

(defn bold
  "Bold the given string"
  [text]
  (string "\x1b[1m" text "\x1b[0m"))

(defn bold-underline
  "Bold and underline the given string"
  [text]
  (bold (underline text)))
  
