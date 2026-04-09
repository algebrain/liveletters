(ns liveletters.frontend-app.core-test
  (:require [cljs.test :refer-macros [deftest is]]
            [liveletters.frontend-app.core :as core]))

(deftest exposes-module-info
  (is (= {:module :liveletters-frontend-app
          :language :cljc}
         (core/module-info))))

(deftest init-loads-feed-sync-and-failures-into-app-state
  (let [calls (atom [])
        adapter {:invoke-command (fn [command payload]
                                   (swap! calls conj [command payload])
                                   (case command
                                     "get_home_feed" [{:post_id "post-1"
                                                       :resource_id "blog-1"
                                                       :author_id "alice"
                                                       :body "First post"
                                                       :hidden false}]
                                     "get_sync_status" {:status "healthy"
                                                        :applied_messages 1
                                                        :duplicate_messages 0
                                                        :malformed_messages 0
                                                        :deferred_events 0
                                                        :pending_outbox 1}
                                     "list_incoming_failures" []
                                     nil))}
        app-state (core/create-app-state)]
    (core/init! adapter app-state)
    (is (= 1 (count (:feed @app-state))))
    (is (= :healthy (get-in @app-state [:sync-status :status])))
    (is (= [] (:incoming-failures @app-state)))
    (is (= [["get_home_feed" nil]
            ["get_sync_status" nil]
            ["list_incoming_failures" nil]]
           @calls))))

(deftest create-post-intent-calls-frontend-api-and-refreshes-feed
  (let [calls (atom [])
        adapter {:invoke-command (fn [command payload]
                                   (swap! calls conj [command payload])
                                   (case command
                                     "create_post" {:ok true}
                                     "get_home_feed" [{:post_id "post-1"
                                                       :resource_id "blog-1"
                                                       :author_id "alice"
                                                       :body "First post"
                                                       :hidden false}]
                                     nil))}
        app-state (core/create-app-state)]
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
            ["get_home_feed" nil]]
           @calls))))

(deftest load-post-thread-updates-route-and-thread
  (let [adapter {:invoke-command (fn [command payload]
                                   (case command
                                     "get_post_thread" {:post {:post_id (:post-id payload)
                                                               :body "First post"
                                                               :hidden false}
                                                        :comments [{:comment_id "comment-1"
                                                                    :body "First comment"
                                                                    :author_id "bob"
                                                                    :parent_comment_id nil}]}
                                     nil))}
        app-state (core/create-app-state)]
    (core/load-post-thread! adapter app-state "post-1")
    (is (= {:page :post :post-id "post-1"} (:route @app-state)))
    (is (= "post-1" (get-in @app-state [:thread :post :post_id])))))

(deftest root-view-renders-current-page-shell
  (let [app-state (core/create-app-state)]
    (reset! app-state {:route {:page :sync}
                       :feed []
                       :thread nil
                       :sync-status {:status :healthy
                                     :applied-messages 1
                                     :duplicate-messages 0
                                     :malformed-messages 0
                                     :deferred-events 0
                                     :pending-outbox 1}
                       :incoming-failures []
                       :create-post {}
                       :ui {}})
    (is (= :main (first (core/root-view app-state))))
    (is (= "Sync"
           (get-in (core/root-view app-state) [3 1 :title])))))
