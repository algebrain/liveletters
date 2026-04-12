(ns liveletters.frontend-app.modals
  "Модальные окна: Settings, Add Subscription."
  (:require [liveletters.frontend-app.theme.modal :as modal]
            [liveletters.frontend-app.theme.settings :as settings]
            [liveletters.frontend-app.store :as app-store]
            [liveletters.ui-kit.core :as ui-kit]
            [liveletters.ui-model.core :as ui-model]))

(def security-options
  [{:value "starttls" :label "STARTTLS"}
   {:value "tls" :label "TLS / SSL"}
   {:value "none" :label "None"}])

(defn- close-icon []
  [:svg {:width "18" :height "18" :viewBox "0 0 24 24"
         :fill "none" :stroke "currentColor" :stroke-width "2"
         :stroke-linecap "round" :stroke-linejoin "round"}
   [:line {:x1 "18" :y1 "6" :x2 "6" :y2 "18"}]
   [:line {:x1 "6" :y1 "6" :x2 "18" :y2 "18"}]])

;; ---------- Settings Modal ----------

(defn- settings-modal-header [closeable? on-close]
  [modal/modal-header {}
   [modal/modal-title {} "Settings"]
   (when closeable?
     [modal/modal-close {:type "button" :on-click on-close}
      (close-icon)])])

(defn- settings-form [store form adapter]
  [settings/settings-layout {:class "ll-settings-form"}
   [settings/settings-grid {:style {:grid-template-columns "repeat(3, minmax(0, 1fr))"}}
    [settings/settings-card {}
     [:h3 {} "Profile"]
     [settings/settings-column
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
    [settings/settings-card {}
     [:h3 {} "SMTP delivery"]
     [settings/settings-column
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
    [settings/settings-card {}
     [:h3 {} "IMAP inbox"]
     [settings/settings-column
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
                                       {:imap-mailbox (.. % -target -value)})}]]]]
   [:div {:style {:display "flex" :justify-content "flex-end" :gap "10px"}}
    [ui-kit/button {:label "Save settings"
                    :on-click #(app-store/submit-settings! adapter store)}]]])

(defn settings-modal [store state closeable? on-close]
  (let [form (ui-model/settings-form-view-model (:settings-form state))
        adapter (get-in state [:runtime :adapter])
        error-message (get-in state [:ui :error :message])]
    [modal/modal-overlay {:class "ll-modal-overlay"}
     [modal/modal-content {:class "ll-modal-content"}
      [settings-modal-header closeable? on-close]
      [modal/modal-body {}
       (when error-message
         [ui-kit/error-state {:message error-message}])
       [settings-form store form adapter]]]]))

;; ---------- Add Subscription Modal ----------

(defn add-subscription-modal [on-close]
  [modal/modal-overlay {:class "ll-modal-overlay"}
   [modal/modal-content {:class "ll-modal-content" :style {:maxWidth "420px"}}
    [modal/modal-header {}
     [modal/modal-title {} "Add Subscription"]
     [modal/modal-close {:type "button" :on-click on-close}
      (close-icon)]]
    [modal/modal-body {}
     [:div {:style {:display "grid" :gap "16px"}}
      [:div {:style {:display "grid" :gap "8px"}}
       [:label {:style {:fontSize "12px" :fontWeight 600 :textTransform "uppercase"
                        :letterSpacing "0.08em" :color "var(--text-secondary)"}}
        "Email Address"]
       [modal/add-sub-input {:type "email"
                             :placeholder "user@example.com"}]]
      [:div {:style {:display "flex" :justifyContent "flex-end"}}
       [modal/add-sub-button {} "Subscribe"]]]]]])
