(ns liveletters.ui-kit.core
  #?(:cljs (:require [reagent.core :as r])))

(defn module-info []
  {:module :liveletters-ui-kit
   :language :cljc})

(defn button [{:keys [label variant on-click disabled?]
               :or {variant :primary
                    disabled? false}}]
  [:button (cond-> {:type "button"
                    :class (str "ll-button ll-button--" (name variant))
                    :disabled disabled?}
             on-click (assoc :on-click on-click))
   label])

(defn text-input [{:keys [label value placeholder on-change]
                   :or {value ""
                        placeholder ""}}]
  [:label {:class "ll-field"}
   [:span {:class "ll-field__label"} label]
   [:input (cond-> {:type "text"
                    :value value
                    :placeholder placeholder
                    :class "ll-input"}
             on-change (assoc :on-change on-change)
             (nil? on-change) (assoc :read-only true))]])

(defn section [{:keys [title children]
                :or {children []}}]
  [:section {:class "ll-section"}
   [:h2 {:class "ll-section__title"} title]
   (into [:div {:class "ll-section__body"}] children)])

(defn loading-state [{:keys [message]}]
  [:div {:class "ll-state ll-state--loading"
         :role "status"
         :aria-live "polite"}
   message])

(defn empty-state [{:keys [message]}]
  [:div {:class "ll-state ll-state--empty"}
   message])

(defn error-state [{:keys [message]}]
  [:div {:class "ll-state ll-state--error"
         :role "alert"}
   message])
