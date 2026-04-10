(ns liveletters.frontend-app.core-test
  (:require [cljs.test :refer-macros [async deftest is]]
            [liveletters.frontend-app.core :as core]))

(deftest exposes-module-info
  (is (= {:module :liveletters-frontend-app
          :language :cljc}
         (core/module-info))))

(deftest init-loads-feed-sync-and-failures-into-app-state
  (let [calls (atom [])
        adapter {:invoke-command (fn [command payload on-success _on-error]
                                   (swap! calls conj [command payload])
                                   (case command
                                     "get_bootstrap_state" (on-success {:setup_completed true})
                                     "get_settings" (on-success {:nickname "alice"
                                                                 :email_address "alice@example.com"
                                     :avatar_url nil
                                     :smtp_host "smtp.example.com"
                                     :smtp_port 587
                                     :smtp_security "starttls"
                                     :smtp_username "alice"
                                     :smtp_password "secret"
                                     :smtp_hello_domain "example.com"
                                     :imap_host "imap.example.com"
                                     :imap_port 143
                                     :imap_security "starttls"
                                     :imap_username "alice"
                                     :imap_password "secret"
                                     :imap_mailbox "INBOX"
                                                                 :setup_completed true})
                                     "get_home_feed" (on-success {:posts [{:post_id "post-1"
                                                                           :resource_id "blog-1"
                                                                           :author_id "alice"
                                                                           :body "First post"
                                                                           :hidden false}]})
                                     "get_sync_status" (on-success {:status "healthy"
                                                                    :applied_messages 1
                                                                    :duplicate_messages 0
                                                                    :replayed_messages 0
                                                                    :unauthorized_messages 0
                                                                    :invalid_messages 0
                                                                    :malformed_messages 0
                                                                    :deferred_events 0
                                                                    :pending_outbox 1})
                                     "list_incoming_failures" (on-success [])
                                     "list_event_failures" (on-success [])
                                     nil))
                 :subscribe-event (fn [_event-name _handler] :subscription-token)}
        app-state (core/create-app-state)]
    (core/init! adapter app-state)
    (is (= {:checked? true :setup-completed? true} (:bootstrap @app-state)))
    (is (= 1 (count (:feed @app-state))))
    (is (= :healthy (get-in @app-state [:sync-status :status])))
    (is (= [] (:incoming-failures @app-state)))
    (is (= [] (:event-failures @app-state)))
    (is (= {:page :feed} (:route @app-state)))
    (is (= [["get_bootstrap_state" nil]
            ["get_settings" nil]
            ["get_home_feed" nil]
            ["get_sync_status" nil]
            ["list_incoming_failures" nil]
            ["list_event_failures" nil]]
           @calls))))

(deftest init_routes_to_initial_setup_when_bootstrap_is_incomplete
  (let [calls (atom [])
        adapter {:invoke-command (fn [command payload on-success _on-error]
                                   (swap! calls conj [command payload])
                                   (case command
                                     "get_bootstrap_state" (on-success {:setup_completed false})
                                     "get_settings" (on-success {:nickname ""
                                                                 :email_address ""
                                     :avatar_url nil
                                     :smtp_host ""
                                     :smtp_port 587
                                     :smtp_security "starttls"
                                     :smtp_username ""
                                     :smtp_password ""
                                     :smtp_hello_domain ""
                                     :imap_host ""
                                     :imap_port 143
                                     :imap_security "starttls"
                                     :imap_username ""
                                     :imap_password ""
                                     :imap_mailbox "INBOX"
                                                                 :setup_completed false})
                                     nil))
                 :subscribe-event (fn [_event-name _handler] :subscription-token)}
        app-state (core/create-app-state)]
    (core/init! adapter app-state)
    (is (= {:page :initial-setup} (:route @app-state)))
    (is (= "" (get-in @app-state [:settings-form :nickname])))
    (is (= [["get_bootstrap_state" nil]
            ["get_settings" nil]]
           @calls))))

(deftest create-post-intent-calls-frontend-api-and-refreshes-feed
  (let [calls (atom [])
        adapter {:invoke-command (fn [command payload on-success _on-error]
                                   (swap! calls conj [command payload])
                                   (case command
                                     "create_post" (on-success {:ok true})
                                     "get_home_feed" (on-success {:posts [{:post_id "post-1"
                                                                           :resource_id "blog-1"
                                                                           :author_id "alice"
                                                                           :body "First post"
                                                                           :hidden false}]})
                                     "get_sync_status" (on-success {:status "healthy"
                                                                    :applied_messages 1
                                                                    :duplicate_messages 0
                                                                    :replayed_messages 0
                                                                    :unauthorized_messages 0
                                                                    :invalid_messages 0
                                                                    :malformed_messages 0
                                                                    :deferred_events 0
                                                                    :pending_outbox 1})
                                     nil))}
        app-state (core/create-app-state)]
    (swap! app-state assoc-in [:runtime :adapter] adapter)
    (core/update-create-post-form! app-state {:post-id "post-1"
                                              :resource-id "blog-1"
                                              :author-id "alice"
                                              :created-at 1
                                              :body "First post"})
    (core/submit-create-post! adapter app-state)
    (is (= "post-1" (get-in @app-state [:feed 0 :post_id])))
    (is (= [["create_post" {:post-id "post-1"
                            :resource-id "blog-1"
                            :author-id "alice"
                            :created-at 1
                            :body "First post"}]
            ["get_home_feed" nil]
            ["get_sync_status" nil]]
           @calls))))

(deftest create-post-intent-fills-runnable-defaults-when-form-is-minimal
  (let [calls (atom [])
        adapter {:invoke-command (fn [command payload on-success _on-error]
                                   (swap! calls conj [command payload])
                                   (case command
                                     "create_post" (on-success {:ok true})
                                     "get_home_feed" (on-success {:posts []})
                                     "get_sync_status" (on-success {:status "healthy"
                                                                    :applied_messages 0
                                                                    :duplicate_messages 0
                                                                    :replayed_messages 0
                                                                    :unauthorized_messages 0
                                                                    :invalid_messages 0
                                                                    :malformed_messages 0
                                                                    :deferred_events 0
                                                                    :pending_outbox 0})
                                     nil))}
        app-state (core/create-app-state)]
    (swap! app-state assoc :create-post {:post-id ""
                                         :resource-id ""
                                         :author-id ""
                                         :created-at 0
                                         :body "Runnable post"})
    (core/submit-create-post! adapter app-state)
    (is (= "create_post" (ffirst @calls)))
    (is (= "blog-1" (get-in @calls [0 1 :resource-id])))
    (is (= "alice" (get-in @calls [0 1 :author-id])))
    (is (pos-int? (get-in @calls [0 1 :created-at])))
    (is (re-matches #"post-\d+" (get-in @calls [0 1 :post-id])))))

(deftest load-post-thread-updates-route-and-thread
  (let [adapter {:invoke-command (fn [command payload on-success _on-error]
                                   (case command
                                     "get_post_thread" (on-success {:post {:post_id (:post-id payload)
                                                                           :body "First post"
                                                                           :hidden false}
                                                                    :comments [{:comment_id "comment-1"
                                                                                :body "First comment"
                                                                                :author_id "bob"
                                                                                :parent_comment_id nil}]})
                                     nil))}
        app-state (core/create-app-state)]
    (core/load-post-thread! adapter app-state "post-1")
    (is (= {:page :post :post-id "post-1"} (:route @app-state)))
    (is (= "post-1" (get-in @app-state [:thread :post :post_id])))))

(deftest sync-status-event-refreshes-diagnostics-state
  (let [sync-handler* (atom nil)
        calls (atom [])
        adapter {:invoke-command (fn [command payload on-success _on-error]
                                   (swap! calls conj [command payload])
                                   (case command
                                     "get_bootstrap_state" (on-success {:setup_completed true})
                                     "get_settings" (on-success {:nickname "alice"
                                                                 :email_address "alice@example.com"
                                                                 :avatar_url nil
                                                                 :smtp_host "smtp.example.com"
                                                                 :smtp_port 587
                                                                 :smtp_security "starttls"
                                                                 :smtp_username "alice"
                                                                 :smtp_password "secret"
                                                                 :smtp_hello_domain "example.com"
                                                                 :imap_host "imap.example.com"
                                                                 :imap_port 143
                                                                 :imap_security "starttls"
                                                                 :imap_username "alice"
                                                                 :imap_password "secret"
                                                                 :imap_mailbox "INBOX"
                                                                 :setup_completed true})
                                     "get_home_feed" (on-success {:posts []})
                                     "get_sync_status" (on-success {:status "degraded"
                                                                    :applied_messages 0
                                                                    :duplicate_messages 0
                                                                    :replayed_messages 1
                                                                    :unauthorized_messages 1
                                                                    :invalid_messages 0
                                                                    :malformed_messages 0
                                                                    :deferred_events 0
                                                                    :pending_outbox 0})
                                     "list_incoming_failures" (on-success [{:message_id "message-1"
                                                                           :status "invalid"
                                                                           :preview "Broken"}])
                                     "list_event_failures" (on-success [{:event_id "event-1"
                                                                        :event_type "comment_edited"
                                                                        :resource_id "blog-1"
                                                                        :apply_status "unauthorized"
                                                                        :failure_reason "actor_cannot_edit_comment"}])
                                     nil))
                 :subscribe-event (fn [event-name handler]
                                    (when (= event-name "sync-status-changed")
                                      (reset! sync-handler* handler))
                                    :subscription-token)}
        app-state (core/create-app-state)]
    (core/init! adapter app-state)
    (@sync-handler* {:reason "sync-status-changed"})
    (is (= :degraded (get-in @app-state [:sync-status :status])))
    (is (= 1 (count (:incoming-failures @app-state))))
    (is (= 1 (count (:event-failures @app-state))))))

(deftest save-settings-intent-updates-bootstrap-and_navigates_from_initial_setup
  (let [calls (atom [])
        adapter {:invoke-command (fn [command payload on-success _on-error]
                                   (swap! calls conj [command payload])
                                   (case command
                                     "save_settings" (on-success {:nickname "alice"
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
                                                                  :setup_completed true})
                                     "get_home_feed" (on-success {:posts []})
                                     "get_sync_status" (on-success {:status "healthy"
                                                                    :applied_messages 0
                                                                    :duplicate_messages 0
                                                                    :replayed_messages 0
                                                                    :unauthorized_messages 0
                                                                    :invalid_messages 0
                                                                    :malformed_messages 0
                                                                    :deferred_events 0
                                                                    :pending_outbox 0})
                                     "list_incoming_failures" (on-success [])
                                     "list_event_failures" (on-success [])
                                     nil))}
        app-state (core/create-app-state)]
    (swap! app-state assoc
           :route {:page :initial-setup}
           :runtime {:adapter adapter}
           :settings-form {:nickname "alice"
                           :email-address "alice@example.com"
                                                                 :avatar-url ""
                                                                 :smtp-host "smtp.example.com"
                                                                 :smtp-port 587
                                                                 :smtp-security "starttls"
                                                                 :smtp-username "alice"
                                                                 :smtp-password "secret"
                                                                 :smtp-hello-domain "example.com"
                                                                 :imap-host "imap.example.com"
                                                                 :imap-port 143
                                                                 :imap-security "starttls"
                                                                 :imap-username "alice"
                                                                 :imap-password "secret"
                                                                 :imap-mailbox "INBOX"})
    (core/submit-settings! adapter app-state)
    (is (= {:checked? true :setup-completed? true} (:bootstrap @app-state)))
    (is (= {:page :feed} (:route @app-state)))
    (is (= "alice" (get-in @app-state [:settings-form :nickname])))
    (is (= "save_settings" (ffirst @calls)))))

(deftest smtp-username-autofills-imap-username-after-delay
  (async done
    (let [app-state (core/create-app-state)]
      (core/update-settings-form! app-state {:smtp-username "alice@example.com"})
      (js/setTimeout
       (fn []
         (is (= "alice@example.com"
                (get-in @app-state [:settings-form :smtp-username])))
         (is (= "alice@example.com"
                (get-in @app-state [:settings-form :imap-username])))
         (done))
       1100))))

(deftest imap-username-autofill-does-not-overwrite-existing-smtp-username
  (async done
    (let [app-state (core/create-app-state)]
      (swap! app-state assoc-in [:settings-form :smtp-username] "smtp-user")
      (core/update-settings-form! app-state {:imap-username "imap-user"})
      (js/setTimeout
       (fn []
         (is (= "smtp-user"
                (get-in @app-state [:settings-form :smtp-username])))
         (is (= "imap-user"
                (get-in @app-state [:settings-form :imap-username])))
         (done))
       1100))))

(deftest root-view-renders-current-page-shell
  (let [app-state (core/create-app-state)]
    (reset! app-state {:route {:page :sync}
                       :bootstrap {:checked? true
                                   :setup-completed? true}
                       :feed []
                       :thread nil
                       :sync-status {:status :healthy
                                     :applied-messages 1
                                     :duplicate-messages 0
                                     :replayed-messages 0
                                     :unauthorized-messages 0
                                     :invalid-messages 0
                                     :malformed-messages 0
                                     :deferred-events 0
                                     :pending-outbox 1}
                       :incoming-failures []
                       :event-failures []
                       :settings-form {}
                       :create-post {}
                       :runtime {:adapter nil}
                       :ui {}})
    (is (vector? (core/root-view app-state)))
    (is (= "Sync"
           (get-in (core/root-view app-state) [2 3 1 :title])))))
