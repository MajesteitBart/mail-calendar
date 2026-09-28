//! A photo fetch that is slow to answer, held open by a test.
//!
//! A provider fetch can take a minute under its retry policy. Nothing the user is waiting for
//! may wait on it: not an account being added or removed, not the account list, and not the
//! write a swipe queued. Each test parks the fetch mid-flight and then asks for one of those.

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use async_trait::async_trait;
use engine_api::{AccountId, CalendarWrites, EmailAddress};
use engine_core::{
    contact::{AddressBook, ContactCard, ContactResource},
    mail::Message,
    sync::SyncState,
};
use engine_provider::{
    ConnectionInfo, ContactPhoto, ContactSourceSync, ContactsProvider, Provider, ProviderResult,
};
use tokio::sync::Notify;

use super::test_app::{FakeContacts, PNG, card_with_photo};
use crate::{ContactsIntent, Intent};

#[allow(clippy::duplicate_mod)]
#[path = "tests_fakes.rs"]
mod fakes;

use fakes::{FakeProvider, account, app, message, msg};

/// Far longer than any of the operations under test takes, and far shorter than a test runner's
/// own limit: a regression fails here with a message rather than hanging the suite.
const PATIENCE: Duration = Duration::from_secs(5);

const ADA: &str = "ada@example.test";

/// [`FakeContacts`] whose photo fetch announces itself and then waits to be released.
struct ParkedContacts {
    inner: FakeContacts,
    entered: Arc<Notify>,
    release: Arc<Notify>,
}

impl Provider for ParkedContacts {
    fn connection_info(&self) -> ConnectionInfo {
        self.inner.connection_info()
    }
}

impl CalendarWrites for ParkedContacts {}

#[async_trait]
impl ContactsProvider for ParkedContacts {
    async fn sync_address_books(
        &self,
        account: &AccountId,
        cursor: Option<&SyncState>,
    ) -> ProviderResult<ContactSourceSync<AddressBook>> {
        self.inner.sync_address_books(account, cursor).await
    }

    async fn sync_contacts(
        &self,
        account: &AccountId,
        cursor: Option<&SyncState>,
    ) -> ProviderResult<ContactSourceSync<ContactCard>> {
        self.inner.sync_contacts(account, cursor).await
    }

    async fn fetch_contact_photo(
        &self,
        account: &AccountId,
        card: &ContactCard,
        media: &ContactResource,
    ) -> ProviderResult<Option<ContactPhoto>> {
        self.entered.notify_one();
        self.release.notified().await;
        self.inner.fetch_contact_photo(account, card, media).await
    }
}

/// The two halves of the gate: the test waits on `entered` and lets the fetch go with `release`.
struct Gate {
    entered: Arc<Notify>,
    release: Arc<Notify>,
}

impl Gate {
    fn new() -> Self {
        Self {
            entered: Arc::new(Notify::new()),
            release: Arc::new(Notify::new()),
        }
    }

    /// A contacts adapter holding Ada's card, whose photo fetch this gate holds.
    fn contacts(&self) -> ParkedContacts {
        ParkedContacts {
            inner: ada(),
            entered: Arc::clone(&self.entered),
            release: Arc::clone(&self.release),
        }
    }
}

fn ada() -> FakeContacts {
    FakeContacts {
        card: card_with_photo(ADA),
        photo: Some(PNG.to_vec()),
    }
}

/// A message in the inbox from Ada, so a list showing it has a face to look up.
fn from_ada(key: &str, subject: &str) -> Message {
    let mut message = message(key, "a", subject);
    message.envelope.from = vec![EmailAddress::new(ADA)];
    message
}

#[tokio::test]
async fn a_photo_fetch_in_flight_does_not_hold_up_the_account_list() {
    let gate = Gate::new();
    let mut work = account("work", FakeProvider::new());
    work.contact_providers = vec![Box::new(gate.contacts())];
    let surfaces = Arc::new(Mutex::new(Vec::new()));
    let app = app(vec![work], &surfaces);
    let home = AccountId::try_from("home").unwrap();

    // Syncing contacts runs the photo pass, which parks in Ada's fetch.
    let pass = app.dispatch(Intent::Contacts(ContactsIntent::RefreshContacts));
    let probe = async {
        gate.entered.notified().await;
        // Adding an account queues a write on the account list, and every read after it waits
        // its turn, so the two are asked together: the switcher redrawing while an account is
        // being added.
        let outcome = tokio::time::timeout(PATIENCE, async {
            tokio::join!(
                app.add_account_deferred(account("home", FakeProvider::new())),
                app.account_rows(),
            );
            app.remove_account(&home).await;
        })
        .await;
        gate.release.notify_one();
        outcome
    };
    let ((), outcome) = tokio::join!(pass, probe);

    assert!(
        outcome.is_ok(),
        "adding and removing an account, and listing the accounts, finish while a photo fetch is \
         still in flight",
    );
    assert_eq!(app.account_rows().await.len(), 1);
}

#[tokio::test]
async fn an_archive_reaches_the_provider_while_a_photo_fetch_is_in_flight() {
    let gate = Gate::new();
    // Ada wrote twice. One message is archived; the other keeps her in the list, so the rebuild
    // that follows the archive has a face to look up.
    let provider = FakeProvider::with_archive(vec![
        from_ada("m1", "Quarterly report"),
        from_ada("m2", "Lunch plans"),
    ]);
    let edits = provider.edits();
    let mut work = account("work", provider);
    work.contact_providers = vec![Box::new(gate.contacts())];
    let surfaces = Arc::new(Mutex::new(Vec::new()));
    let app = app(vec![work], &surfaces);
    let id = AccountId::try_from("work").unwrap();

    // Mail first, while no card exists: the pass records "nobody" without fetching anything.
    app.dispatch(Intent::RefreshMail).await;
    // Then Ada's card, straight into the store so no pass runs, and the earlier "nobody"
    // forgotten, which is what a contacts sync does. Her photo is now one fetch away.
    let cards = ada();
    app.engine.sync_address_books(&cards, &id).await.unwrap();
    app.engine.sync_contact_cards(&cards, &id).await.unwrap();
    app.forget_sender_photos();

    let archive = app.dispatch(Intent::Archive {
        message: msg("work", "m1"),
    });
    let probe = async {
        let parked = tokio::time::timeout(PATIENCE, gate.entered.notified()).await;
        let written = edits.lock().unwrap().len();
        gate.release.notify_one();
        (parked, written)
    };
    let ((), (parked, written)) = tokio::join!(archive, probe);

    assert!(parked.is_ok(), "a photo fetch was started for Ada");
    assert_eq!(
        written, 1,
        "the archive reached the provider before any photo fetch, not after it",
    );
}
