(ns liveletters.frontend-app.pages
  (:require [liveletters.ui-kit.core :as ui-kit]
            [liveletters.ui-model.core :as ui-model]))

(defn feed-page [state]
  (let [feed (ui-model/feed-view-model (:feed state))
        form (:create-post state)]
    [ui-kit/section
     {:title "Home feed"
      :children
      [[:div {:class "ll-compose"}
        [ui-kit/text-input {:label "Post body"
                            :value (:body form)
                            :placeholder "Write your post"}]
        [ui-kit/button {:label "Create post"}]]
       (if (:empty? feed)
         [ui-kit/empty-state {:message "Nothing here yet"}]
         [:ul {:class "ll-feed"}
          (for [item (:items feed)]
            ^{:key (:id item)}
            [:li {:class "ll-feed__item"}
             [:h3 {} (:title item)]
             [:p {} (:meta item)]
             (when (:hidden? item)
               [:span {:class "ll-feed__flag"} "Hidden"])])])]}]))

(defn post-page [state]
  (let [thread (ui-model/post-thread-view-model (:thread state))]
    [ui-kit/section
     {:title "Post thread"
      :children
      [[:article {:class "ll-post"}
        [:h3 {} (get-in thread [:post :title])]
        (when (get-in thread [:post :hidden?])
          [:span {:class "ll-post__flag"} "Hidden"])]
       [:div {:class "ll-comment-form"}
        [ui-kit/text-input {:label "Comment"
                            :value ""
                            :placeholder "Write a comment"}]
        [ui-kit/button {:label "Add comment" :variant :secondary}]]
       [:ul {:class "ll-thread"}
        (for [comment (:comments thread)]
          ^{:key (:id comment)}
          [:li {:class "ll-thread__item"}
           [:p {} (:body comment)]
           [:small {} (:author comment)]])]]}]))

(defn sync-page [state]
  (let [sync-status (ui-model/sync-status-view-model (:sync-status state))]
    [ui-kit/section
     {:title "Sync"
      :children
      [[:div {:class (str "ll-sync ll-sync--" (name (:tone sync-status)))}
        [:strong {} (:label sync-status)]
        [:ul {}
         [:li {} (str "Applied: " (get-in sync-status [:details :applied]))]
         [:li {} (str "Duplicates: " (get-in sync-status [:details :duplicates]))]
         [:li {} (str "Replays: " (get-in sync-status [:details :replays]))]
         [:li {} (str "Unauthorized: " (get-in sync-status [:details :unauthorized]))]
         [:li {} (str "Invalid: " (get-in sync-status [:details :invalid]))]
         [:li {} (str "Malformed: " (get-in sync-status [:details :malformed]))]
         [:li {} (str "Deferred: " (get-in sync-status [:details :deferred]))]
         [:li {} (str "Outbox: " (get-in sync-status [:details :outbox]))]]]]}]))

(defn diagnostics-page [state]
  (let [failures (ui-model/incoming-failures-view-model (:incoming-failures state))
        event-failures (ui-model/event-failures-view-model (:event-failures state))]
    [ui-kit/section
     {:title "Diagnostics"
      :children
      [(if (and (empty? failures) (empty? event-failures))
         [ui-kit/empty-state {:message "No diagnostic failures"}]
         [:div {:class "ll-diagnostics"}
          (when-not (empty? failures)
            [:<>
             [:h3 {} "Incoming failures"]
             [:ul {:class "ll-failures"}
              (for [failure failures]
                ^{:key (:id failure)}
                [:li {:class "ll-failures__item"}
                 [:strong {} (name (:kind failure))]
                 [:p {} (:preview failure)]])]])
          (when-not (empty? event-failures)
            [:<>
             [:h3 {} "Event failures"]
             [:ul {:class "ll-event-failures"}
              (for [failure event-failures]
                ^{:key (:id failure)}
                [:li {:class "ll-event-failures__item"}
                 [:strong {} (name (:kind failure))]
                 [:p {} (:event-type failure)]
                 [:small {} (:reason failure)]])]])])]}]))
