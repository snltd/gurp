(def defaults
  (table
    :directories @{:group "root"
                   :mode "0755"
                   :owner "root"
                   :recurse false}

    :files @{:group "root"
             :mode "0755"
             :owner "root"}))
