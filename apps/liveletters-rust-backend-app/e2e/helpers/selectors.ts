/**
 * Селекторы для e2e-тестов LiveLetters.
 *
 * Используются CSS-классы из текущего UI.
 * После добавления data-testid — обновить этот файл.
 */

export const initialSetupPage = {
  /** Заголовок страницы initial setup */
  heading: "h2",
  /** Кнопка "Save and continue" */
  saveButton: "button.ll-button",
};

export const settingsPage = {
  /** Заголовок страницы настроек */
  heading: "h2",
  /** Кнопка "Save settings" */
  saveButton: "button.ll-button",
};

/** Основные поля формы настроек */
export const settingsForm = {
  /** Контейнер всей формы */
  container: ".ll-settings-form",

  // --- Profile card ---
  nickname: 'input[placeholder="alice"]',
  email: 'input[placeholder="alice@example.com"]',
  avatarUrl: 'input[placeholder="https://example.com/avatar.png"]',

  // --- SMTP card ---
  smtpHost: 'input[placeholder="smtp.example.com"]',
  smtpPort: 'input[placeholder="587"]',
  smtpUsername: '[placeholder="alice"]',
  smtpPassword: 'input[type="password"]',
  smtpHelloDomain: 'input[placeholder="example.com"]',

  // --- IMAP card ---
  imapHost: 'input[placeholder="imap.example.com"]',
  imapPort: 'input[placeholder="143"]',
  imapUsername: '[placeholder="alice"]',
  imapMailbox: 'input[placeholder="INBOX"]',
};

/** Индикаторы post-setup состояния */
export const postSetupIndicators = {
  mainWindowTitle: "LiveLetters",
};
