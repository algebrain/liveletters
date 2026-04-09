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
     :message "unexpected backend error"
     :details nil}))

(defn invoke-command [adapter command payload]
  ((:invoke-command adapter) command payload))

(defn subscribe-event [adapter event-name handler]
  ((:subscribe-event adapter) event-name handler))

(defn create-post [adapter request]
  (invoke-command adapter "create_post" request))

(defn get-home-feed [adapter]
  (invoke-command adapter "get_home_feed" nil))

(defn get-post-thread [adapter request]
  (invoke-command adapter "get_post_thread" request))

(defn get-sync-status [adapter]
  (some-> (invoke-command adapter "get_sync_status" nil)
          sync-status-dto))

(defn list-incoming-failures [adapter]
  (invoke-command adapter "list_incoming_failures" nil))

(defn list-event-failures [adapter]
  (some->> (invoke-command adapter "list_event_failures" nil)
           (mapv event-failure-dto)))

(defn subscribe-sync-status-changed [adapter handler]
  (subscribe-event adapter "sync-status-changed" handler))
