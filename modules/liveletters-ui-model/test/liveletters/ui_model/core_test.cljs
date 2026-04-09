(ns liveletters.ui-model.core-test
  (:require [cljs.test :refer-macros [deftest is]]
            [liveletters.ui-model.core :as core]))

(deftest exposes-module-info
  (is (= {:module :liveletters-ui-model
          :language :cljc}
         (core/module-info))))

(deftest builds-feed-view-model
  (is (= {:items [{:id "post-2"
                   :title "Second post"
                   :meta "alice in blog-1"
                   :hidden? false}
                  {:id "post-1"
                   :title "First post"
                   :meta "alice in blog-1"
                   :hidden? true}]
          :empty? false}
         (core/feed-view-model
          [{:post_id "post-1"
            :resource_id "blog-1"
            :author_id "alice"
            :body "First post"
            :hidden true}
           {:post_id "post-2"
            :resource_id "blog-1"
            :author_id "alice"
            :body "Second post"
            :hidden false}]))))

(deftest builds-empty-feed-view-model
  (is (= {:items []
          :empty? true}
         (core/feed-view-model []))))

(deftest builds-post-thread-view-model
  (is (= {:post {:id "post-1"
                 :title "First post"
                 :hidden? false}
          :comments [{:id "comment-1"
                      :body "First comment"
                      :author "bob"
                      :reply-to nil}
                     {:id "comment-2"
                      :body "Reply"
                      :author "alice"
                      :reply-to "comment-1"}]
          :comment-count 2}
         (core/post-thread-view-model
          {:post {:post_id "post-1"
                  :body "First post"
                  :hidden false}
           :comments [{:comment_id "comment-1"
                       :body "First comment"
                       :author_id "bob"
                       :parent_comment_id nil}
                      {:comment_id "comment-2"
                       :body "Reply"
                       :author_id "alice"
                       :parent_comment_id "comment-1"}]}))))

(deftest formats-sync-status-for-ui
  (is (= {:label "Healthy"
          :tone :positive
          :details {:applied 3
                    :duplicates 1
                    :replays 2
                    :unauthorized 1
                    :invalid 4
                    :malformed 0
                    :deferred 0
                    :outbox 2}}
         (core/sync-status-view-model
          {:status :healthy
           :applied-messages 3
           :duplicate-messages 1
           :replayed-messages 2
           :unauthorized-messages 1
           :invalid-messages 4
           :malformed-messages 0
           :deferred-events 0
           :pending-outbox 2}))))

(deftest normalizes-incoming-failures-for-screen
  (is (= [{:id "message-1"
           :kind :malformed
           :preview "bad message"}]
         (core/incoming-failures-view-model
          [{:message_id "message-1"
            :status "malformed"
            :preview "bad message"}]))))

(deftest normalizes-event-failures-for-screen
  (is (= [{:id "event-1"
           :event-type "comment_edited"
           :resource-id "blog-1"
           :kind :unauthorized
           :reason "actor_cannot_edit_comment"}]
         (core/event-failures-view-model
          [{:event-id "event-1"
            :event-type "comment_edited"
            :resource-id "blog-1"
            :apply-status :unauthorized
            :failure-reason "actor_cannot_edit_comment"}]))))
