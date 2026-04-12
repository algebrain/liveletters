/**
 * Mock Tauri API для e2e-тестов.
 *
 * Внедряется через page.addInitScript() ДО загрузки frontend-бандла.
 * Имитирует window.__TAURI_INTERNALS__ и window.__TAURI__,
 * которые использует @tauri-apps/api для invoke/emit/listen.
 */

const TAURI_MOCK_SCRIPT = `
(function() {
  // Хранилище фейковых настроек (имитирует SQLite backend)
  let storedSettings = null;
  let setupCompleted = false;
  let eventListeners = {};
  let nextId = 1;
  let callbacks = {};

  // Tauri invoke mock
  async function invoke(command, args) {
    console.log('[tauri-mock] invoke:', command, args);

    switch (command) {
      case 'plugin:event|listen':
        // Mock для подписки на события — регистрируем handler
        return Promise.resolve(args?.handler ?? 0);

      case 'plugin:event|emit':
        // Mock для эмита событий
        return Promise.resolve(null);

      case 'get_bootstrap_state':
        return { setup_completed: setupCompleted };

      case 'get_settings':
        if (!storedSettings) {
          // Возвращаем пустые настройки вместо ошибки — форма будет пустой
          return {
            nickname: '',
            email_address: '',
            avatar_url: null,
            smtp_host: '',
            smtp_port: 587,
            smtp_security: 'STARTTLS',
            smtp_username: '',
            smtp_password: '',
            smtp_hello_domain: '',
            imap_host: '',
            imap_port: 143,
            imap_security: 'STARTTLS',
            imap_username: '',
            imap_password: '',
            imap_mailbox: 'INBOX',
            setup_completed: false,
          };
        }
        return storedSettings;

      case 'save_settings': {
        const req = args?.request || {};
        storedSettings = {
          nickname: req.nickname || '',
          email_address: req.email_address || '',
          avatar_url: req.avatar_url || null,
          smtp_host: req.smtp_host || '',
          smtp_port: req.smtp_port || 587,
          smtp_security: req.smtp_security || 'STARTTLS',
          smtp_username: req.smtp_username || '',
          smtp_password: req.smtp_password || '',
          smtp_hello_domain: req.smtp_hello_domain || '',
          imap_host: req.imap_host || '',
          imap_port: req.imap_port || 143,
          imap_security: req.imap_security || 'STARTTLS',
          imap_username: req.imap_username || '',
          imap_password: req.imap_password || '',
          imap_mailbox: req.imap_mailbox || 'INBOX',
          setup_completed: true,
        };
        setupCompleted = true;

        // Эмитим событие sync-status-changed после сохранения
        emitEvent('sync-status-changed', { reason: 'sync-status-changed' });
        emitEvent('feed-updated', { reason: 'feed-updated' });

        return storedSettings;
      }

      case 'get_home_feed':
        return { posts: [] };

      case 'create_post':
        return null;

      case 'get_post_thread':
        return { post: null, comments: [] };

      case 'get_sync_status':
        return {
          status: 'idle',
          applied_messages: 0,
          duplicate_messages: 0,
          replayed_messages: 0,
          unauthorized_messages: 0,
          invalid_messages: 0,
          malformed_messages: 0,
          deferred_events: 0,
          pending_outbox: 0,
        };

      case 'list_incoming_failures':
        return [];

      case 'list_event_failures':
        return [];

      case 'log_frontend_error':
        return null;

      default:
        throw new Error('Unknown command: ' + command);
    }
  }

  function transformCallback(callback, once) {
    const id = nextId++;
    callbacks[id] = { callback, once };
    return id;
  }

  function unregisterCallback(id) {
    delete callbacks[id];
  }

  // Internal event emission (for sync-status-changed, feed-updated)
  function emitEvent(eventName, payload) {
    const listeners = eventListeners[eventName] || [];
    listeners.forEach(function(l) {
      try {
        l.callback({ event: eventName, payload: payload });
      } catch(e) {
        console.error('[tauri-mock] event handler error:', e);
      }
      if (l.once) {
        delete callbacks[l.id];
      }
    });
  }

  // Tauri event listen mock
  function listen(event, handler) {
    if (!eventListeners[event]) {
      eventListeners[event] = [];
    }
    const id = nextId++;
    eventListeners[event].push({ id, callback: handler });
    return Promise.resolve({
      unsubscribe: function() {
        eventListeners[event] = eventListeners[event].filter(function(l) {
          return l.id !== id;
        });
      }
    });
  }

  function emit(event, payload) {
    const listeners = eventListeners[event] || [];
    listeners.forEach(function(l) {
      try {
        l.callback({ event: event, payload: payload });
      } catch(e) {
        console.error('[tauri-mock] event handler error:', e);
      }
    });
  }

  // Expose Tauri internals — именно здесь @tauri-apps/api ищет invoke
  window.__TAURI_INTERNALS__ = {
    invoke: invoke,
    transformCallback: transformCallback,
    unregisterCallback: unregisterCallback,
    event: {
      listen: listen,
      emit: emit,
    },
  };

  window.__TAURI__ = {
    core: {
      invoke: invoke,
    },
    event: {
      listen: listen,
      emit: emit,
    },
  };

  // Экспорт для тестов
  window.__TAURI_MOCK__ = {
    getSettings: function() { return storedSettings; },
    isSetupCompleted: function() { return setupCompleted; },
    reset: function() {
      storedSettings = null;
      setupCompleted = false;
      eventListeners = {};
      callbacks = {};
    },
    setSetupCompleted: function(val) { setupCompleted = val; },
    emitEvent: emitEvent,
  };
})();
`;

export { TAURI_MOCK_SCRIPT };
