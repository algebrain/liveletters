(ns liveletters.ui-kit.core-test
  (:require [cljs.test :refer-macros [deftest is]]
            [liveletters.ui-kit.core :as core]))

(deftest exposes-module-info
  (is (= {:module :liveletters-ui-kit
          :language :cljc}
         (core/module-info))))

(deftest button-renders-stable-hiccup-shape
  (is (= [:button {:type "button"
                   :class "ll-button ll-button--primary"}
          "Create post"]
         (core/button {:label "Create post"}))))

(deftest input-renders-label-and-value
  (is (= [:label {:class "ll-field"}
          [:span {:class "ll-field__label"} "Title"]
          [:input {:type "text"
                   :value "Hello"
                   :placeholder "Write title"
                   :class "ll-input"
                   :read-only true}]]
         (core/text-input {:label "Title"
                           :value "Hello"
                           :placeholder "Write title"}))))

(deftest input-renders-on-change-when-field-is-editable
  (let [handler (fn [_event] nil)]
    (is (= [:label {:class "ll-field"}
            [:span {:class "ll-field__label"} "Body"]
            [:input {:type "text"
                     :value "Editable"
                     :placeholder ""
                     :class "ll-input"
                     :on-change handler}]]
           (core/text-input {:label "Body"
                             :value "Editable"
                             :on-change handler})))))

(deftest section-wraps-title-and-children
  (is (= [:section {:class "ll-section"}
          [:h2 {:class "ll-section__title"} "Feed"]
          [:div {:class "ll-section__body"}
           [:p {} "Item"]]]
         (core/section {:title "Feed"
                        :children [[:p {} "Item"]]}))))

(deftest state-components-have_a11y_markers
  (is (= [:div {:class "ll-state ll-state--loading"
                :role "status"
                :aria-live "polite"}
          "Loading feed"]
         (core/loading-state {:message "Loading feed"})))
  (is (= [:div {:class "ll-state ll-state--empty"}
          "Nothing here yet"]
         (core/empty-state {:message "Nothing here yet"})))
  (is (= [:div {:class "ll-state ll-state--error"
                :role "alert"}
          "Sync failed"]
         (core/error-state {:message "Sync failed"}))))
