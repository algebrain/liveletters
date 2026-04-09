(ns liveletters.ui-model.core)

(defn module-info []
  {:module :liveletters-ui-model
   :language :cljc})

(defn feed-view-model [posts]
  {:items (->> posts
               (sort-by :post_id >)
               (mapv (fn [post]
                       {:id (:post_id post)
                        :title (:body post)
                        :meta (str (:author_id post) " in " (:resource_id post))
                        :hidden? (boolean (:hidden post))})))
   :empty? (empty? posts)})

(defn post-thread-view-model [thread]
  {:post {:id (get-in thread [:post :post_id])
          :title (get-in thread [:post :body])
          :hidden? (boolean (get-in thread [:post :hidden]))}
   :comments (mapv (fn [comment]
                     {:id (:comment_id comment)
                      :body (:body comment)
                      :author (:author_id comment)
                      :reply-to (:parent_comment_id comment)})
                   (:comments thread))
   :comment-count (count (:comments thread))})

(defn sync-status-view-model [sync-status]
  {:label (case (:status sync-status)
            :healthy "Healthy"
            :degraded "Degraded"
            "Unknown")
   :tone (case (:status sync-status)
           :healthy :positive
           :degraded :warning
           :neutral)
   :details {:applied (:applied-messages sync-status)
             :duplicates (:duplicate-messages sync-status)
             :malformed (:malformed-messages sync-status)
             :deferred (:deferred-events sync-status)
             :outbox (:pending-outbox sync-status)}})

(defn incoming-failures-view-model [failures]
  (mapv (fn [failure]
          {:id (:message_id failure)
           :kind (keyword (:status failure))
           :preview (:preview failure)})
        failures))
