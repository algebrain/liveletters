(ns liveletters.frontend-app.pages
  (:require [liveletters.frontend-app.store :as app-store]
            [liveletters.frontend-app.theme.core :as theme]
            [liveletters.ui-kit.core :as ui-kit]
            [liveletters.ui-model.core :as ui-model]))

(def security-options
  [{:value "starttls" :label "STARTTLS"}
   {:value "tls" :label "TLS / SSL"}
   {:value "none" :label "None"}])

(defn- settings-fields [store form]
  [[theme/settings-layout {:class "ll-settings-form"}
    [theme/settings-grid
     [theme/settings-card
      [:h3 {} "Profile"]
      [theme/settings-column
       [ui-kit/text-input {:label "Nickname"
                           :value (:nickname form)
                           :placeholder "alice"
                           :on-change #(app-store/update-settings-form!
                                        store
                                        {:nickname (.. % -target -value)})}]
       [ui-kit/text-input {:label "Email"
                           :value (:email-address form)
                           :placeholder "alice@example.com"
                           :on-change #(app-store/update-settings-form!
                                        store
                                        {:email-address (.. % -target -value)})}]
       [ui-kit/text-input {:label "Avatar URL"
                           :value (:avatar-url form)
                           :placeholder "https://example.com/avatar.png"
                           :on-change #(app-store/update-settings-form!
                                        store
                                        {:avatar-url (.. % -target -value)})}]]]
     [theme/settings-card
      [:h3 {} "SMTP delivery"]
      [theme/settings-column
       [ui-kit/text-input {:label "SMTP host"
                           :value (:smtp-host form)
                           :placeholder "smtp.example.com"
                           :on-change #(app-store/update-settings-form!
                                        store
                                        {:smtp-host (.. % -target -value)})}]
       [ui-kit/text-input {:label "SMTP port"
                           :value (str (:smtp-port form))
                           :placeholder "587"
                           :on-change #(app-store/update-settings-form!
                                        store
                                        {:smtp-port (.. % -target -value)})}]
       [ui-kit/select-input {:label "SMTP security"
                             :value (:smtp-security form)
                             :options security-options
                             :on-change #(app-store/update-settings-form!
                                          store
                                          {:smtp-security (.. % -target -value)})}]
       [ui-kit/text-input {:label "SMTP username"
                           :value (:smtp-username form)
                           :placeholder "alice"
                           :on-change #(app-store/update-settings-form!
                                        store
                                        {:smtp-username (.. % -target -value)})}]
       [ui-kit/password-input {:label "SMTP password"
                               :value (:smtp-password form)
                               :on-change #(app-store/update-settings-form!
                                            store
                                            {:smtp-password (.. % -target -value)})}]
       [ui-kit/text-input {:label "SMTP hello domain"
                           :value (:smtp-hello-domain form)
                           :placeholder "example.com"
                           :on-change #(app-store/update-settings-form!
                                        store
                                        {:smtp-hello-domain (.. % -target -value)})}]]]
     [theme/settings-card
      [:h3 {} "IMAP inbox"]
      [theme/settings-column
       [ui-kit/text-input {:label "IMAP host"
                           :value (:imap-host form)
                           :placeholder "imap.example.com"
                           :on-change #(app-store/update-settings-form!
                                        store
                                        {:imap-host (.. % -target -value)})}]
       [ui-kit/text-input {:label "IMAP port"
                           :value (str (:imap-port form))
                           :placeholder "143"
                           :on-change #(app-store/update-settings-form!
                                        store
                                        {:imap-port (.. % -target -value)})}]
       [ui-kit/select-input {:label "IMAP security"
                             :value (:imap-security form)
                             :options security-options
                             :on-change #(app-store/update-settings-form!
                                          store
                                          {:imap-security (.. % -target -value)})}]
       [ui-kit/text-input {:label "IMAP username"
                           :value (:imap-username form)
                           :placeholder "alice"
                           :on-change #(app-store/update-settings-form!
                                        store
                                        {:imap-username (.. % -target -value)})}]
       [ui-kit/password-input {:label "IMAP password"
                               :value (:imap-password form)
                               :on-change #(app-store/update-settings-form!
                                            store
                                            {:imap-password (.. % -target -value)})}]
       [ui-kit/text-input {:label "IMAP mailbox"
                           :value (:imap-mailbox form)
                           :placeholder "INBOX"
                           :on-change #(app-store/update-settings-form!
                                        store
                                        {:imap-mailbox (.. % -target -value)})}]]]]]])

(defn initial-setup-page [store state]
  (let [form (ui-model/settings-form-view-model (:settings-form state))
        form-status (ui-model/settings-form-status form)
        adapter (get-in state [:runtime :adapter])
        error-message (get-in state [:ui :error :message])]
    [ui-kit/section
     {:title "Initial setup"
      :children
      (into
       [[theme/page-copy {}
         "Set your local profile and mail connection settings before the first sync. The same form becomes your permanent settings page later."]
        [theme/actions-row {}
         [ui-kit/button {:label "Save and continue"
                         :disabled? (or (nil? adapter)
                                        (not (:submittable? form-status)))
                         :on-click #(app-store/submit-settings! adapter store)}]]]
       (concat
        (when error-message
          [[ui-kit/error-state {:message error-message}]])
        (settings-fields store form)))}]))

(defn settings-page [store state]
  (let [form (ui-model/settings-form-view-model (:settings-form state))
        form-status (ui-model/settings-form-status form)
        adapter (get-in state [:runtime :adapter])
        error-message (get-in state [:ui :error :message])]
    [ui-kit/section
     {:title "Settings"
      :children
      (into
       [[theme/page-copy {}
         "Update your public profile and the local mail transport that powers sync and delivery."]
        [theme/actions-row {}
         [ui-kit/button {:label "Save settings"
                         :disabled? (or (nil? adapter)
                                        (not (:submittable? form-status)))
                         :on-click #(app-store/submit-settings! adapter store)}]]]
       (concat
        (when error-message
          [[ui-kit/error-state {:message error-message}]])
        (settings-fields store form)))}]))

(defn feed-page [store state]
  (let [feed (ui-model/feed-view-model (:feed state))
        form (:create-post state)
        adapter (get-in state [:runtime :adapter])]
    [ui-kit/section
     {:title "Home feed"
      :children
      [[:div {:class "ll-compose"}
        [ui-kit/text-input {:label "Post body"
                            :value (:body form)
                            :placeholder "Write your post"
                            :on-change #(app-store/update-create-post-form!
                                         store
                                         {:body (.. % -target -value)})}]
        [ui-kit/button {:label "Create post"
                        :disabled? (or (nil? adapter) (empty? (:body form)))
                        :on-click #(app-store/submit-create-post! adapter store)}]]
       (if (:empty? feed)
         [ui-kit/empty-state {:message "Nothing here yet"}]
         [:ul {:class "ll-feed"}
          (for [item (:items feed)]
            ^{:key (:id item)}
            [:li {:class "ll-feed__item"
                  :on-click #(app-store/load-post-thread! adapter store (:id item))}
             [:h3 {} (:title item)]
             [:p {} (:meta item)]
             (when (:hidden? item)
               [:span {:class "ll-feed__flag"} "Hidden"])])])]}]))

(defn post-page [_store state]
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

(defn sync-page [_store state]
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

(defn diagnostics-page [_store state]
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
