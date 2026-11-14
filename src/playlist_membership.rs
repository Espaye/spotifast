//! Which of the account's own playlists hold each song, so a heart can
//! tell a song saved in any playlist from one saved nowhere and its
//! checklist can tick every playlist that holds it. Liked Songs is not
//! here: the saved map answers for it.
//!
//! Each editable playlist is read once over the playback session, one
//! request per thousand rows, and again when the library reports a new
//! snapshot for it. Edits made here change the index at once and hold
//! back reads of that playlist until Spotify confirms them, so an answer
//! from before an edit cannot undo it.

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

/// Reads on their way at once. The playback session's background reads
/// share four slots, so two stay free for the page on screen.
const READS_AT_ONCE: usize = 2;
/// How long a playlist whose read failed waits before it is asked again.
const RETRY_AFTER: Duration = Duration::from_secs(60);

#[derive(Default)]
pub(crate) struct Membership {
    lists: HashMap<String, List>,
    /// How many lists hold each song.
    anywhere: HashMap<String, u32>,
    /// Reads on their way, by playlist id.
    reading: HashMap<String, Reading>,
    /// Edits sent and not yet answered, by playlist id.
    writing: HashMap<String, u32>,
    failed: HashMap<String, Instant>,
}

#[derive(Default)]
struct List {
    /// The library snapshot the rows were read at.
    snapshot: Option<String>,
    /// How many rows hold each song.
    counts: HashMap<String, u32>,
    /// Every row is known, so a song missing from `counts` is not there.
    complete: bool,
}

struct Reading {
    snapshot: Option<String>,
    /// An edit since the read was asked makes its answer out of date.
    stale: bool,
}

impl Membership {
    fn wants(&self, id: &str, snapshot: Option<&str>) -> bool {
        !self.reading.contains_key(id)
            && self.writing.get(id).is_none_or(|count| *count == 0)
            && self
                .failed
                .get(id)
                .is_none_or(|at| at.elapsed() >= RETRY_AFTER)
            && !self.lists.get(id).is_some_and(|list| {
                list.complete && list.snapshot.as_deref() == snapshot
            })
    }

    /// Takes the account's editable playlists (id and library snapshot),
    /// forgets those no longer among them, and returns the ids to read
    /// now, marked as being read.
    pub(crate) fn start_reads(&mut self, playlists: &[(String, Option<String>)]) -> Vec<String> {
        let editable: HashSet<&str> = playlists.iter().map(|(id, _)| id.as_str()).collect();
        let gone: Vec<String> = self
            .lists
            .keys()
            .filter(|id| !editable.contains(id.as_str()))
            .cloned()
            .collect();
        for id in gone {
            self.forget(&id);
        }
        let mut started = Vec::new();
        for (id, snapshot) in playlists {
            if self.reading.len() >= READS_AT_ONCE {
                break;
            }
            if self.wants(id, snapshot.as_deref()) {
                self.reading.insert(
                    id.clone(),
                    Reading {
                        snapshot: snapshot.clone(),
                        stale: false,
                    },
                );
                started.push(id.clone());
            }
        }
        started
    }

    fn forget(&mut self, id: &str) {
        if let Some(list) = self.lists.remove(id) {
            for uri in list.counts.keys() {
                self.lost(uri);
            }
        }
    }

    fn lost(&mut self, uri: &str) {
        if let Some(count) = self.anywhere.get_mut(uri) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                self.anywhere.remove(uri);
            }
        }
    }

    /// Takes a read's answer: every song in the playlist `id`.
    pub(crate) fn receive(&mut self, id: &str, result: Result<Vec<String>, String>) {
        let Some(reading) = self.reading.remove(id) else {
            return;
        };
        match result {
            Ok(uris) => {
                self.failed.remove(id);
                if reading.stale {
                    return;
                }
                let mut counts = HashMap::new();
                for uri in uris {
                    *counts.entry(uri).or_insert(0u32) += 1;
                }
                self.forget(id);
                for uri in counts.keys() {
                    *self.anywhere.entry(uri.clone()).or_insert(0) += 1;
                }
                self.lists.insert(
                    id.to_string(),
                    List {
                        snapshot: reading.snapshot,
                        counts,
                        complete: true,
                    },
                );
            }
            Err(error) => {
                log::debug!("could not read which songs a playlist holds: {error}");
                self.failed.insert(id.to_string(), Instant::now());
            }
        }
    }

    /// Whether the playlist `id` holds `uri`; `None` until it has been read.
    pub(crate) fn holds(&self, id: &str, uri: &str) -> Option<bool> {
        let list = self.lists.get(id)?;
        if list.counts.contains_key(uri) {
            Some(true)
        } else {
            list.complete.then_some(false)
        }
    }

    /// Whether any of the account's playlists is known to hold `uri`.
    pub(crate) fn holds_anywhere(&self, uri: &str) -> bool {
        self.anywhere.contains_key(uri)
    }

    /// An edit of the playlist `id` is on its way: reads wait for it.
    pub(crate) fn edit_started(&mut self, id: &str) {
        *self.writing.entry(id.to_string()).or_insert(0) += 1;
        if let Some(reading) = self.reading.get_mut(id) {
            reading.stale = true;
        }
    }

    /// Spotify confirmed an edit of the playlist `id`, which is now at
    /// `snapshot`. The edit is already in the index, so the new snapshot
    /// needs no read of its own.
    pub(crate) fn edit_confirmed(&mut self, id: &str, snapshot: Option<String>) {
        self.edit_answered(id);
        if let Some(list) = self.lists.get_mut(id)
            && list.complete
        {
            list.snapshot = snapshot;
        }
    }

    /// An edit of the playlist `id` failed, which leaves the index unsure:
    /// the playlist is read again.
    pub(crate) fn edit_failed(&mut self, id: &str) {
        self.edit_answered(id);
        if let Some(list) = self.lists.get_mut(id) {
            list.complete = false;
        }
    }

    fn edit_answered(&mut self, id: &str) {
        if let Some(count) = self.writing.get_mut(id) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                self.writing.remove(id);
            }
        }
    }

    /// Songs added to the playlist `id` here, one row each.
    pub(crate) fn add<'a>(&mut self, id: &str, uris: impl IntoIterator<Item = &'a str>) {
        self.edit_started(id);
        let list = self.lists.entry(id.to_string()).or_default();
        for uri in uris {
            let count = list.counts.entry(uri.to_string()).or_insert(0);
            *count += 1;
            if *count == 1 {
                *self.anywhere.entry(uri.to_string()).or_insert(0) += 1;
            }
        }
    }

    /// Every row of each song taken out of the playlist `id` here.
    pub(crate) fn remove_all<'a>(&mut self, id: &str, uris: impl IntoIterator<Item = &'a str>) {
        self.edit_started(id);
        let Some(list) = self.lists.get_mut(id) else {
            return;
        };
        let mut lost = Vec::new();
        for uri in uris {
            if list.counts.remove(uri).is_some() {
                lost.push(uri);
            }
        }
        for uri in lost {
            self.lost(uri);
        }
    }

    /// One row per entry taken out of the playlist `id` here; other rows
    /// of the same song stay.
    pub(crate) fn remove_rows<'a>(&mut self, id: &str, uris: impl IntoIterator<Item = &'a str>) {
        self.edit_started(id);
        let Some(list) = self.lists.get_mut(id) else {
            return;
        };
        let mut lost = Vec::new();
        for uri in uris {
            if let Some(count) = list.counts.get_mut(uri) {
                *count = count.saturating_sub(1);
                if *count == 0 {
                    list.counts.remove(uri);
                    lost.push(uri);
                }
            }
        }
        for uri in lost {
            self.lost(uri);
        }
    }
}
