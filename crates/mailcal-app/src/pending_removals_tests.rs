//! The hide predicate on its own. The loop tests (`tests_mail_actions`, `tests_selection`,
//! `tests_report`) prove each write path reaches it.

use std::sync::Arc;

use engine_api::{AccountId, MailListRow, Message, ProviderKey};
use engine_core::{
    ids::{MailboxId, MessageId},
    membership::Memberships,
};

use super::PendingRemovals;

fn account(id: &str) -> AccountId {
    AccountId::try_from(id).unwrap()
}

fn key(key: &str) -> ProviderKey {
    ProviderKey::new(key).unwrap()
}

fn mailbox(id: &str) -> MailboxId {
    MailboxId::try_from(id).unwrap()
}

fn message(key: &str, mailbox_id: &str) -> Message {
    Message::new(
        MessageId::try_from(key).unwrap(),
        Memberships::of_one(mailbox(mailbox_id)),
    )
}

fn row(account_id: &str, key: &str, mailbox_id: &str) -> Arc<MailListRow> {
    Arc::new(MailListRow::project(
        &account(account_id),
        &message(key, mailbox_id),
    ))
}

#[test]
fn a_moved_row_is_hidden_until_it_is_filed_in_its_destination() {
    let mut pending = PendingRemovals::default();
    pending.hide(&account("acct-1"), &key("m1"), Some(&mailbox("archive")));

    assert!(
        pending.hides(&row("acct-1", "m1", "inbox")),
        "not moved yet"
    );
    assert!(
        !pending.hides(&row("acct-1", "m1", "archive")),
        "the same key filed in its destination is the move having landed",
    );
    assert!(
        !pending.hides(&row("acct-2", "m1", "inbox")),
        "a key is unique only within its account",
    );
}

#[test]
fn a_permanent_delete_stays_hidden_until_its_key_is_gone() {
    let mut pending = PendingRemovals::default();
    pending.hide(&account("acct-1"), &key("m1"), None);

    assert!(pending.hides(&row("acct-1", "m1", "archive")));
    pending.settle(&[account("acct-1")], &[row("acct-1", "m1", "inbox")]);
    assert!(!pending.is_empty(), "still in the store, so still hidden");
    pending.settle(&[account("acct-1")], &[]);
    assert!(
        pending.is_empty(),
        "gone from the store, so the hide has done its job"
    );
}

#[test]
fn settling_judges_only_the_accounts_that_were_read() {
    let mut pending = PendingRemovals::default();
    let archive = mailbox("archive");
    for hidden in ["arrived", "waiting", "gone"] {
        pending.hide(&account("acct-1"), &key(hidden), Some(&archive));
    }
    pending.hide(&account("acct-2"), &key("elsewhere"), Some(&archive));

    pending.settle(
        &[account("acct-1")],
        &[
            row("acct-1", "arrived", "archive"),
            row("acct-1", "waiting", "inbox"),
        ],
    );

    assert!(!pending.hides(&row("acct-1", "arrived", "inbox")));
    assert!(pending.hides(&row("acct-1", "waiting", "inbox")));
    assert!(!pending.hides(&row("acct-1", "gone", "inbox")));
    assert!(
        pending.hides(&row("acct-2", "elsewhere", "inbox")),
        "an account that was not read says nothing about its hides",
    );
}

#[test]
fn a_streamed_commit_settles_what_it_removed_or_refiled() {
    let mut pending = PendingRemovals::default();
    let archive = mailbox("archive");
    for hidden in ["removed", "refiled", "untouched"] {
        pending.hide(&account("acct-1"), &key(hidden), Some(&archive));
    }

    pending.settle_commit(
        &account("acct-1"),
        &[message("refiled", "archive"), message("untouched", "inbox")],
        &[key("removed")],
    );

    assert!(!pending.hides(&row("acct-1", "removed", "inbox")));
    assert!(!pending.hides(&row("acct-1", "refiled", "inbox")));
    assert!(pending.hides(&row("acct-1", "untouched", "inbox")));
}
