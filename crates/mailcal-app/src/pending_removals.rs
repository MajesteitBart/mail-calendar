//! Rows taken off the list the instant the user archives, deletes or reports them, before any
//! sync has shown where the message went.

use std::{
    collections::HashMap,
    sync::{Arc, MutexGuard},
};

use engine_api::{AccountId, MailListRow, Message, Provider, ProviderKey};
use engine_core::ids::MailboxId;

use crate::App;

/// The optimistic hides, each with the mailbox its message was sent to.
///
/// A hide ends once the store shows the write landed, and what that looks like depends on the
/// transport. IMAP files the moved copy under a new key, so the old key leaves the store. JMAP,
/// Gmail and Graph keep the key and change the mailboxes the row is filed in. A row is therefore
/// hidden only while it is **not yet** filed in its destination: a hide that waited for the key
/// to go would keep a JMAP message out of the very Archive it was archived to.
#[derive(Debug, Clone, Default)]
pub(crate) struct PendingRemovals {
    /// Per account, per message key, where the message was sent. `None` is a permanent delete,
    /// which has no destination and ends only when the key does.
    by_account: HashMap<AccountId, HashMap<ProviderKey, Option<MailboxId>>>,
}

impl PendingRemovals {
    /// Whether `row` stays off the list: hidden, and not yet filed where it was sent.
    pub(crate) fn hides(&self, row: &MailListRow) -> bool {
        self.by_account
            .get(&row.account)
            .and_then(|keys| keys.get(&row.mail.key))
            .is_some_and(|destination| !filed_in(row.mailboxes.iter(), destination.as_ref()))
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.by_account.is_empty()
    }

    /// Drops the hides the loaded `rows` settle, for the `accounts` they were read from: a key
    /// no longer among them has left, and a row filed in its destination has arrived. An account
    /// that was not read says nothing about its hides.
    pub(crate) fn settle(&mut self, accounts: &[AccountId], rows: &[Arc<MailListRow>]) {
        let loaded: HashMap<(&AccountId, &ProviderKey), &MailListRow> = rows
            .iter()
            .map(|row| ((&row.account, &row.mail.key), row.as_ref()))
            .collect();
        for account in accounts {
            self.retain_in(account, |key, destination| {
                loaded
                    .get(&(account, key))
                    .is_some_and(|row| !filed_in(row.mailboxes.iter(), destination.as_ref()))
            });
        }
    }

    /// Drops the hides one streamed commit settles: the keys it removed, and the messages it
    /// delivered already filed in their destination.
    pub(crate) fn settle_commit(
        &mut self,
        account: &AccountId,
        upserted: &[Message],
        removed: &[ProviderKey],
    ) {
        self.retain_in(account, |key, destination| {
            !removed.contains(key)
                && !upserted.iter().any(|message| {
                    message.id.key() == key
                        && filed_in(message.mailboxes.iter(), destination.as_ref())
                })
        });
    }

    pub(crate) fn forget_account(&mut self, account: &AccountId) {
        self.by_account.remove(account);
    }

    fn hide(&mut self, account: &AccountId, key: &ProviderKey, destination: Option<&MailboxId>) {
        self.by_account
            .entry(account.clone())
            .or_default()
            .insert(key.clone(), destination.cloned());
    }

    fn retain_in(
        &mut self,
        account: &AccountId,
        keep: impl FnMut(&ProviderKey, &mut Option<MailboxId>) -> bool,
    ) {
        if let Some(keys) = self.by_account.get_mut(account) {
            keys.retain(keep);
            if keys.is_empty() {
                self.by_account.remove(account);
            }
        }
    }
}

/// Whether `mailboxes` include `destination`. A permanent delete has none to arrive in.
fn filed_in<'a>(
    mut mailboxes: impl Iterator<Item = &'a MailboxId>,
    destination: Option<&MailboxId>,
) -> bool {
    destination.is_some_and(|destination| mailboxes.any(|mailbox| mailbox == destination))
}

impl<P: Provider> App<P> {
    /// Takes `keys` off the list now, each until it is filed in `destination`, or until it is
    /// gone when there is none.
    pub(crate) fn hide_optimistically(
        &self,
        account: &AccountId,
        keys: &[ProviderKey],
        destination: Option<&MailboxId>,
    ) {
        let mut pending = self.lock_hides();
        for key in keys {
            pending.hide(account, key, destination);
        }
    }

    /// Puts one row back, for a write the provider refused.
    pub(crate) fn restore_row(&self, account: &AccountId, key: &ProviderKey) {
        self.lock_hides()
            .retain_in(account, |hidden, _| hidden != key);
    }

    /// A copy of the hides, for a filter that runs across an `.await`.
    pub(crate) fn pending_hides(&self) -> PendingRemovals {
        self.lock_hides().clone()
    }

    pub(crate) fn lock_hides(&self) -> MutexGuard<'_, PendingRemovals> {
        self.pending_removals
            .lock()
            .expect("pending-removals mutex poisoned")
    }
}

#[cfg(test)]
#[path = "pending_removals_tests.rs"]
mod tests;
