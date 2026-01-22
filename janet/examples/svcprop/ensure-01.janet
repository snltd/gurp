(svcprop/ensure "mariadb"
                :properties {:application/datadir "/data"
                             :application/active true
                             :application/timeout 50})
