(ns liveletters.frontend-api.core-test
  (:require [cljs.test :refer-macros [deftest is]]
            [liveletters.frontend-api.core :as core]))

(deftest exposes-module-info
  (is (= {:module :liveletters-frontend-api
          :language :cljc}
         (core/module-info))))

(deftest create-post-command-is-forwarded-through-adapter
  (let [calls (atom [])
        adapter {:invoke-command (fn [command payload]
                                   (swap! calls conj [command payload])
                                   {:ok true})}
        request {:post-id "post-1"
                 :resource-id "blog-1"
                 :author-id "alice"
                 :created-at 1
                 :body "First post"}]
    (is (= {:ok true}
           (core/create-post adapter request)))
    (is (= [["create_post" request]]
           @calls))))

(deftest query-functions-use-query-command-names
  (let [calls (atom [])
        adapter {:invoke-command (fn [command payload]
                                   (swap! calls conj [command payload])
                                   {:items []})}]
    (is (= {:items []}
           (core/get-home-feed adapter)))
    (is (= {:items []}
           (core/get-post-thread adapter {:post-id "post-1"})))
    (is (= [["get_home_feed" nil]
            ["get_post_thread" {:post-id "post-1"}]]
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
        adapter {:subscribe-event (fn [event-name event-handler]
                                    (swap! calls conj [event-name event-handler])
                                    :subscription-token)}]
    (is (= :subscription-token
           (core/subscribe-sync-status-changed adapter handler)))
    (is (= [["sync-status-changed" handler]]
           @calls))))

(deftest dto-shapes-are-stable
  (is (= {:post-id "post-1"
          :resource-id "blog-1"
          :author-id "alice"
          :created-at 1
          :body "First post"}
         (core/create-post-request "post-1" "blog-1" "alice" 1 "First post")))
  (is (= {:status :healthy
          :applied-messages 3
          :duplicate-messages 1
          :malformed-messages 0
          :deferred-events 0
          :pending-outbox 2}
         (core/sync-status-dto {:status "healthy"
                                :applied_messages 3
                                :duplicate_messages 1
                                :malformed_messages 0
                                :deferred_events 0
                                :pending_outbox 2}))))
