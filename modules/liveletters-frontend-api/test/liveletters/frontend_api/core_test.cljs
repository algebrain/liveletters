(ns liveletters.frontend-api.core-test
  (:require [cljs.test :refer-macros [deftest is]]
            [liveletters.frontend-api.core :as core]))

(deftest exposes-module-info
  (is (= {:module :liveletters-frontend-api
          :language :cljc}
         (core/module-info))))

(deftest create-post-command-is-forwarded-through-adapter
  (let [calls (atom [])
        adapter {:invoke-command (fn [command payload on-success _on-error]
                                   (swap! calls conj [command payload])
                                   (on-success {:ok true}))}
        request {:post-id "post-1"
                 :resource-id "blog-1"
                 :author-id "alice"
                 :created-at 1
                 :body "First post"}]
    (core/create-post! adapter request identity identity)
    (is (= [["create_post" request]]
           @calls))))

(deftest settings-commands-use-settings-command-names-and-dto-shapes
  (let [calls (atom [])
        bootstrap (atom nil)
        settings (atom nil)
        saved (atom nil)]
    (core/get-bootstrap-state!
     {:invoke-command (fn [command payload on-success _on-error]
                        (swap! calls conj [command payload])
                        (on-success {:setup_completed true}))}
     #(reset! bootstrap %)
     identity)
    (core/get-settings!
     {:invoke-command (fn [command payload on-success _on-error]
                        (swap! calls conj [command payload])
                        (on-success {:nickname "alice"
                                     :email_address "alice@example.com"
                                     :avatar_url "https://example.com/avatar.png"
                                     :smtp_host "smtp.example.com"
                                     :smtp_port 587
                                     :smtp_username "alice"
                                     :smtp_password "secret"
                                     :smtp_hello_domain "example.com"
                                     :imap_host "imap.example.com"
                                     :imap_port 143
                                     :imap_username "alice"
                                     :imap_password "secret"
                                     :imap_mailbox "INBOX"
                                     :setup_completed true}))}
     #(reset! settings %)
     identity)
    (core/save-settings!
     {:invoke-command (fn [command payload on-success _on-error]
                        (swap! calls conj [command payload])
                        (on-success {:nickname (:nickname payload)
                                     :email_address (:email-address payload)
                                     :avatar_url (:avatar-url payload)
                                     :smtp_host (:smtp-host payload)
                                     :smtp_port (:smtp-port payload)
                                     :smtp_username (:smtp-username payload)
                                     :smtp_password (:smtp-password payload)
                                     :smtp_hello_domain (:smtp-hello-domain payload)
                                     :imap_host (:imap-host payload)
                                     :imap_port (:imap-port payload)
                                     :imap_username (:imap-username payload)
                                     :imap_password (:imap-password payload)
                                     :imap_mailbox (:imap-mailbox payload)
                                     :setup_completed true}))}
     {:nickname "alice"
      :email-address "alice@example.com"
      :avatar-url nil
      :smtp-host "smtp.example.com"
      :smtp-port 587
      :smtp-username "alice"
      :smtp-password "secret"
      :smtp-hello-domain "example.com"
      :imap-host "imap.example.com"
      :imap-port 143
      :imap-username "alice"
      :imap-password "secret"
      :imap-mailbox "INBOX"}
     #(reset! saved %)
     identity)
    (is (= {:setup-completed? true} @bootstrap))
    (is (= "alice" (:nickname @settings)))
    (is (= 587 (:smtp-port @saved)))
    (is (= [["get_bootstrap_state" nil]
            ["get_settings" nil]
            ["save_settings" {:nickname "alice"
                              :email-address "alice@example.com"
                              :avatar-url nil
                              :smtp-host "smtp.example.com"
                              :smtp-port 587
                              :smtp-username "alice"
                              :smtp-password "secret"
                              :smtp-hello-domain "example.com"
                              :imap-host "imap.example.com"
                              :imap-port 143
                              :imap-username "alice"
                              :imap-password "secret"
                              :imap-mailbox "INBOX"}]]
           @calls))))

(deftest query-functions-use-query-command-names
  (let [calls (atom [])
        adapter {:invoke-command (fn [command payload on-success _on-error]
                                   (swap! calls conj [command payload])
                                   (on-success {:posts []}))}
        post-thread (atom nil)
        event-failures (atom nil)]
    (core/get-home-feed! adapter identity identity)
    (core/get-post-thread! {:invoke-command (fn [command payload on-success _on-error]
                                              (swap! calls conj [command payload])
                                              (on-success {:post {:post_id "post-1"}
                                                           :comments []}))}
                           {:post-id "post-1"}
                           #(reset! post-thread %)
                           identity)
    (core/list-event-failures!
     {:invoke-command (fn [command payload on-success _on-error]
                        (swap! calls conj [command payload])
                        (on-success [{:event_id "event-1"
                                      :event_type "comment_edited"
                                      :resource_id "blog-1"
                                      :apply_status "unauthorized"
                                      :failure_reason "actor_cannot_edit_comment"}]))}
     #(reset! event-failures %)
     identity)
    (is (= {:post {:post_id "post-1"} :comments []}
           @post-thread))
    (is (= [{:event-id "event-1"
             :event-type "comment_edited"
             :resource-id "blog-1"
             :apply-status :unauthorized
             :failure-reason "actor_cannot_edit_comment"}]
           @event-failures))
    (is (= [["get_home_feed" nil]
            ["get_post_thread" {:post-id "post-1"}]
            ["list_event_failures" nil]]
           @calls))))

(deftest backend-errors-are-normalized-for-ui
  (is (= {:type :validation
          :message "blank body"
          :details {:field :body}}
         (core/normalize-error {:code "validation_error"
                                :message "blank body"
                                :details {:field :body}})))
  (is (= {:type :unknown
          :message "unexpected backend error"
          :details nil}
         (core/normalize-error nil))))

(deftest event-subscription-layer-is-mockable
  (let [calls (atom [])
        handler (fn [_event] :ok)
        emitted (atom [])
        adapter {:subscribe-event (fn [event-name event-handler]
                                    (swap! calls conj [event-name event-handler])
                                    :subscription-token)
                 :emit-event (fn [event-name payload]
                               (swap! emitted conj [event-name payload])
                               (js/Promise.resolve true))}]
    (is (= :subscription-token
           (core/subscribe-sync-status-changed! adapter handler)))
    (core/emit-event! adapter "frontend-error" {:kind "manual"})
    (is (= [["sync-status-changed" handler]]
           @calls))
    (is (= [["frontend-error" {:kind "manual"}]]
           @emitted))))

(deftest dto-shapes-are-stable
  (is (= {:post-id "post-1"
          :resource-id "blog-1"
          :author-id "alice"
          :created-at 1
          :body "First post"}
         (core/create-post-request "post-1" "blog-1" "alice" 1 "First post")))
  (is (= {:setup-completed? true}
         (core/bootstrap-state-dto {:setup_completed true})))
  (is (= {:nickname "alice"
          :email-address "alice@example.com"
          :avatar-url nil
          :smtp-host "smtp.example.com"
          :smtp-port 587
          :smtp-username "alice"
          :smtp-password "secret"
          :smtp-hello-domain "example.com"
          :imap-host "imap.example.com"
          :imap-port 143
          :imap-username "alice"
          :imap-password "secret"
          :imap-mailbox "INBOX"
          :setup-completed? true}
         (core/settings-dto {:nickname "alice"
                             :email_address "alice@example.com"
                             :avatar_url nil
                             :smtp_host "smtp.example.com"
                             :smtp_port 587
                             :smtp_username "alice"
                             :smtp_password "secret"
                             :smtp_hello_domain "example.com"
                             :imap_host "imap.example.com"
                             :imap_port 143
                             :imap_username "alice"
                             :imap_password "secret"
                             :imap_mailbox "INBOX"
                             :setup_completed true})))
  (is (= {:nickname "alice"
          :email-address "alice@example.com"
          :avatar-url nil
          :smtp-host "smtp.example.com"
          :smtp-port 587
          :smtp-username "alice"
          :smtp-password "secret"
          :smtp-hello-domain "example.com"
          :imap-host "imap.example.com"
          :imap-port 143
          :imap-username "alice"
          :imap-password "secret"
          :imap-mailbox "INBOX"}
         (core/save-settings-request {:nickname "alice"
                                      :email-address "alice@example.com"
                                      :avatar-url nil
                                      :smtp-host "smtp.example.com"
                                      :smtp-port 587
                                      :smtp-username "alice"
                                      :smtp-password "secret"
                                      :smtp-hello-domain "example.com"
                                      :imap-host "imap.example.com"
                                      :imap-port 143
                                      :imap-username "alice"
                                      :imap-password "secret"
                                      :imap-mailbox "INBOX"})))
  (is (= {:status :healthy
          :applied-messages 3
          :duplicate-messages 1
          :replayed-messages 2
          :unauthorized-messages 1
          :invalid-messages 4
          :malformed-messages 0
          :deferred-events 0
          :pending-outbox 2}
         (core/sync-status-dto {:status "healthy"
                                :applied_messages 3
                                :duplicate_messages 1
                                :replayed_messages 2
                                :unauthorized_messages 1
                                :invalid_messages 4
                                :malformed_messages 0
                                :deferred_events 0
                                :pending_outbox 2}))))
