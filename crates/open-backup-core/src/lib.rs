//! open-backup コア: 指定フォルダを利用者自身の GitHub 非公開リポジトリへ自動バックアップする。
//! OS 非依存のロジックのみを持つ(監視・UI は各 OS 側)。
pub mod ai;
pub mod auth;
pub mod config;
pub mod github;
pub mod index;
pub mod sync;
