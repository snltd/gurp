(import ./user-helpers :as lib :only [compact])

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

(defn description-wrapper
  "Returns an array of strings. The first element has the given leader right-
  aligned, along with the first words of 'words'. The remaining elements are
  the rest of 'words', left padded so they form a neat column"
  [leader words pad-width whole-width]
  (print "leader: " leader)
  (print "words " words)
  (print "pad-width " pad-width)
  (print "whole-width " whole-width)
  (let [pad (string/repeat " " pad-width)
        raw-words (array ;(lib/compact (string/split " " words)) nil)
        coll @[]
        format-string (string "%" (- pad-width 2) "s ")]

    (var acc (bold (string/format format-string leader)))

    (loop [w :in raw-words]
      (if (or (nil? w) (> (+ (length w) (length acc)) whole-width))
        (do
          (array/push coll acc)
          (set acc (string pad w)))
        (set acc (string acc " " w))))
    coll))
