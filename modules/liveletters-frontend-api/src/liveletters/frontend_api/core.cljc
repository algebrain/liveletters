(ns liveletters.frontend-api.core)

(defn module-info []
  {:module :liveletters-frontend-api
   :language :cljc})

(defn create-post-request [post-id resource-id author-id created-at body]
  {:post-id post-id
   :resource-id resource-id
   :author-id author-id
   :created-at created-at
   :body body})

(defn bootstrap-state-dto [backend-response]
  {:setup-completed? (boolean (:setup_completed backend-response))})

(defn settings-dto [backend-response]
  {:nickname (:nickname backend-response)
   :email-address (:email_address backend-response)
   :avatar-url (:avatar_url backend-response)
   :smtp-host (:smtp_host backend-response)
   :smtp-port (:smtp_port backend-response)
   :smtp-username (:smtp_username backend-response)
   :smtp-password (:smtp_password backend-response)
   :smtp-hello-domain (:smtp_hello_domain backend-response)
   :imap-host (:imap_host backend-response)
   :imap-port (:imap_port backend-response)
   :imap-username (:imap_username backend-response)
   :imap-password (:imap_password backend-response)
   :imap-mailbox (:imap_mailbox backend-response)
   :setup-completed? (boolean (:setup_completed backend-response))})

(defn save-settings-request [form]
  {:nickname (:nickname form)
   :email-address (:email-address form)
   :avatar-url (:avatar-url form)
   :smtp-host (:smtp-host form)
   :smtp-port (:smtp-port form)
   :smtp-username (:smtp-username form)
   :smtp-password (:smtp-password form)
   :smtp-hello-domain (:smtp-hello-domain form)
   :imap-host (:imap-host form)
   :imap-port (:imap-port form)
   :imap-username (:imap-username form)
   :imap-password (:imap-password form)
   :imap-mailbox (:imap-mailbox form)})

(defn sync-status-dto [backend-response]
  {:status (keyword (:status backend-response))
   :applied-messages (:applied_messages backend-response)
   :duplicate-messages (:duplicate_messages backend-response)
   :replayed-messages (:replayed_messages backend-response)
   :unauthorized-messages (:unauthorized_messages backend-response)
   :invalid-messages (:invalid_messages backend-response)
   :malformed-messages (:malformed_messages backend-response)
   :deferred-events (:deferred_events backend-response)
   :pending-outbox (:pending_outbox backend-response)})

(defn event-failure-dto [backend-response]
  {:event-id (:event_id backend-response)
   :event-type (:event_type backend-response)
   :resource-id (:resource_id backend-response)
   :apply-status (keyword (:apply_status backend-response))
   :failure-reason (:failure_reason backend-response)})

(defn normalize-error [backend-error]
  (case (:code backend-error)
    "validation_error"
    {:type :validation
     :message (:message backend-error)
     :details (:details backend-error)}

    "not_found"
    {:type :not-found
     :message (:message backend-error)
     :details (:details backend-error)}

    {:type :unknown
     :message (or (:message backend-error) "unexpected backend error")
     :details (or (:details backend-error) (:code backend-error))}))

(defn invoke-command! [adapter command payload on-success on-error]
  ((:invoke-command adapter) command payload on-success on-error))

(defn subscribe-event! [adapter event-name handler]
  ((:subscribe-event adapter) event-name handler))

(defn emit-event! [adapter event-name payload]
  ((:emit-event adapter) event-name payload))

(defn create-post! [adapter request on-success on-error]
  (invoke-command! adapter "create_post" request on-success on-error))

(defn get-home-feed! [adapter on-success on-error]
  (invoke-command! adapter "get_home_feed" nil
                   (fn [response]
                     (on-success (or (:posts response) [])))
                   on-error))

(defn get-bootstrap-state! [adapter on-success on-error]
  (invoke-command! adapter "get_bootstrap_state" nil
                   (fn [response]
                     (on-success (some-> response bootstrap-state-dto)))
                   on-error))

(defn get-settings! [adapter on-success on-error]
  (invoke-command! adapter "get_settings" nil
                   (fn [response]
                     (on-success (some-> response settings-dto)))
                   on-error))

(defn save-settings! [adapter request on-success on-error]
  (invoke-command! adapter "save_settings" request
                   (fn [response]
                     (on-success (some-> response settings-dto)))
                   on-error))

(defn get-post-thread! [adapter request on-success on-error]
  (invoke-command! adapter "get_post_thread" request on-success on-error))

(defn get-sync-status! [adapter on-success on-error]
  (invoke-command! adapter "get_sync_status" nil
                   (fn [response]
                     (on-success (some-> response sync-status-dto)))
                   on-error))

(defn list-incoming-failures! [adapter on-success on-error]
  (invoke-command! adapter "list_incoming_failures" nil on-success on-error))

(defn list-event-failures! [adapter on-success on-error]
  (invoke-command! adapter "list_event_failures" nil
                   (fn [response]
                     (on-success (some->> response
                                          (mapv event-failure-dto))))
                   on-error))

(defn subscribe-sync-status-changed! [adapter handler]
  (subscribe-event! adapter "sync-status-changed" handler))

(defn subscribe-feed-updated! [adapter handler]
  (subscribe-event! adapter "feed-updated" handler))
