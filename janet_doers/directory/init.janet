(defn is [path & opts]
  (let [opts (table ;opts)]
    (print "configure directory" path)
    (if-let [comment (get opts :comment)] (print comment))))



