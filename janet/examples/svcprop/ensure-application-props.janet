(svcprop/ensure "example/svc_1"
                :on-change "restart"
                :property-groups {:application "application"}
                :properties {:application/datadir "/data"
                             :application/active true
                             :application/timeout 50})
