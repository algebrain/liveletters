(ns liveletters.frontend-app.state)

(defn initial-state []
  {:route {:page :initial-setup}
   :bootstrap {:checked? false
               :setup-completed? false}
   :feed []
   :thread nil
   :sync-status nil
   :incoming-failures []
   :event-failures []
   :settings-form {:nickname ""
                   :email-address ""
                   :avatar-url ""
                   :smtp-host ""
                   :smtp-port 587
                   :smtp-security "starttls"
                   :smtp-username ""
                   :smtp-password ""
                   :smtp-hello-domain ""
                   :imap-host ""
                   :imap-port 143
                   :imap-security "starttls"
                   :imap-username ""
                   :imap-password ""
                   :imap-mailbox "INBOX"}
   :create-post {:post-id ""
                 :resource-id "blog-1"
                 :author-id "alice"
                 :created-at 0
                 :body ""}
   :runtime {:adapter nil}
   :ui {:loading? false
        :error nil}})
