(svc/ensure "important/service"
            :state "enabled"
            :restarted-by [:/test-role/file/stub])
