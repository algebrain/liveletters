(ns liveletters.frontend-api.tauri-test
  (:require [cljs.test :refer-macros [deftest is]]
            [liveletters.frontend-api.tauri :as tauri]))

(deftest command-args-wraps-request-commands-under-request-key
  (is (= {:request {:smtp-security "starttls"
                    :imap-security "tls"}}
         (tauri/command-args "save_settings"
                             {:smtp-security "starttls"
                              :imap-security "tls"})))
  (is (= {:post-id "post-1"}
         (tauri/command-args "get_post_thread" {:post-id "post-1"}))))

(deftest encode-payload-recursively-snake-cases-nested-request-maps
  (is (= {"request" {"smtp_security" "starttls"
                     "imap_security" "tls"}}
         (tauri/encode-payload
          {:request {:smtp-security "starttls"
                     :imap-security "tls"}}))))

(deftest normalize-invoke-error-preserves-backend-message-from-map
  (is (= {:type :validation
          :message "nickname must not be blank"
          :details "nickname"}
         (tauri/normalize-invoke-error
          #js {:code "validation_error"
               :message "nickname must not be blank"
               :details "nickname"}))))

(deftest normalize-invoke-error-handles-string-rejections
  (is (= {:type :unknown
          :message "permission denied"
          :details "invoke_error"}
         (tauri/normalize-invoke-error "permission denied"))))

(deftest normalize-invoke-error-handles-js-error-objects
  (let [error (js/Error. "bridge exploded")]
    (is (= :unknown (:type (tauri/normalize-invoke-error error))))
    (is (= "bridge exploded" (:message (tauri/normalize-invoke-error error))))
    (is (string? (:details (tauri/normalize-invoke-error error))))))
