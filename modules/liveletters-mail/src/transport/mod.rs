#[cfg(feature = "network")]
pub mod imap;
#[cfg(feature = "network")]
pub mod smtp;

#[cfg(feature = "network")]
pub use imap::ConfiguredImapMailbox;
#[cfg(feature = "network")]
pub use smtp::ConfiguredSmtpTransport;
