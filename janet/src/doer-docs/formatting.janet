(import ./lib :only [join-words])

(defn underline [& words]
  (string "\x1b[4m" (lib/join-words ;words) "\x1b[0m"))

(defn bold [& words]
  (string "\x1b[1m" (lib/join-words ;words) "\x1b[0m"))

(defn bold-underline [& words]
  (bold (underline ;words)))
