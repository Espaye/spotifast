//! One session-local membership source for playlist pickers and hearts.
use std::collections::{HashMap, HashSet};

use crate::api::models::{Page, PlayableItem, PlaylistItem};
use crate::app::App;
use crate::backend::{ApiRequest, ApiResult};

#[derive(Default)]
pub(crate) struct Membership {
    account: Option<String>,
    playlists: HashMap<String, Songs>,
}

#[derive(Default)]
struct Songs {
    snapshot: Option<String>,
    source: Option<(u64, u64)>,
    uris: HashSet<String>,
    removed: HashSet<String>,
    complete: bool,
    next: u32,
    pending: Option<(u64, u32)>,
    failed: bool,
}

impl Membership {
    pub(crate) fn contains(&self, id: &str, uri: &str) -> Option<bool> {
        let songs = self.playlists.get(id)?;
        if songs.removed.contains(uri) {
            Some(false)
        } else if songs.uris.contains(uri) {
            Some(true)
        } else {
            songs.complete.then_some(false)
        }
    }

    pub(crate) fn invalidate(&mut self, id: &str) {
        if let Some(songs) = self.playlists.get_mut(id) {
            songs.pending = None;
            songs.complete = false;
            songs.next = 0;
            songs.failed = false;
            songs.source = None;
        }
    }

    pub(crate) fn add(&mut self, id: &str, items: &[PlayableItem]) {
        let songs = self.playlists.entry(id.to_string()).or_default();
        for item in items {
            songs.removed.remove(item.uri());
            songs.uris.insert(item.uri().to_string());
        }
    }

    pub(crate) fn remove(&mut self, id: &str, uris: &[String]) {
        let songs = self.playlists.entry(id.to_string()).or_default();
        for uri in uris {
            songs.uris.remove(uri);
            songs.removed.insert(uri.clone());
        }
    }
}

impl App {
    /// Run before views temporarily take their pages out of the app to draw.
    pub(crate) fn sync_playlist_membership(&mut self) {
        let account = self.user_id().map(str::to_owned);
        if self.playlist_membership.account != account {
            self.playlist_membership = Membership { account, ..Default::default() };
        }
        let editable = self.editable_playlists();
        self.playlist_membership.playlists.retain(|id, _| editable.iter().any(|(known, _)| known == id));
        for (id, _) in editable {
            let snapshot = self.library.playlists.get().and_then(|all| all.iter().find(|p| p.id == id)).and_then(|p| p.snapshot_id.clone());
            let songs = self.playlist_membership.playlists.entry(id.clone()).or_default();
            if songs.snapshot != snapshot && !self.playlist_pages.get(&id).is_some_and(|page| page.pending_writes > 0) {
                // A library refresh with a different snapshot invalidates old reads.
                *songs = Songs { snapshot, ..Default::default() };
            }
            let Some(page) = self.playlist_pages.get(&id) else { continue };
            if !page.items.loaded_once && page.pending_writes == 0 && page.local_additions.is_empty() && page.items.windows.is_empty() {
                continue;
            }
            let source = (page.generation, page.items.revision);
            if songs.source == Some(source) { continue }
            songs.source = Some(source);
            if page.items.is_complete() {
                songs.uris.clear();
                songs.complete = true;
                songs.pending = None;
            }
            songs.uris.extend(page.items.items.iter().chain(page.items.windows.values().flatten()).filter_map(PlaylistItem::playable).map(|item| item.uri().to_string()));
            songs.uris.extend(page.local_additions.iter().cloned());
            songs.uris.retain(|uri| !songs.removed.contains(uri));
        }
    }

    pub(crate) fn playlist_contains_song(&self, id: &str, uri: &str) -> Option<bool> {
        self.playlist_membership.contains(id, uri)
    }

    /// Read missing membership on the runtime, two pages at a time globally.
    /// Repeated frames share requests. Failures stay unknown, without retry loops.
    pub(crate) fn check_playlist_membership(&mut self) {
        self.sync_playlist_membership();
        if !self.network_reachable { return }
        let active = self.playlist_membership.playlists.values().filter(|songs| songs.pending.is_some()).count();
        let mut slots = 2_usize.saturating_sub(active);
        for (id, _) in self.editable_playlists() {
            if slots == 0 { break }
            if self.playlist_pages.get(&id).is_some_and(|page| page.pending_writes > 0) { continue }
            let songs = self.playlist_membership.playlists.get_mut(&id).unwrap();
            if songs.complete || songs.failed || songs.pending.is_some() { continue }
            self.load_generation = self.load_generation.wrapping_add(1);
            let generation = self.load_generation;
            let offset = songs.next;
            songs.pending = Some((generation, offset));
            self.backend.api(ApiRequest::PlaylistMembership { id, offset, generation });
            slots -= 1;
        }
    }

    pub(crate) fn receive_playlist_membership(&mut self, id: &str, offset: u32, generation: u64, result: ApiResult<Page<PlaylistItem>>) {
        self.sync_playlist_membership();
        let Some(songs) = self.playlist_membership.playlists.get_mut(id) else { return };
        if songs.pending != Some((generation, offset)) { return }
        songs.pending = None;
        match result {
            Ok(page) => {
                songs.uris.extend(page.items.iter().filter_map(PlaylistItem::playable).map(|item| item.uri().to_string()));
                songs.uris.retain(|uri| !songs.removed.contains(uri));
                match page.next_offset() {
                    Some(next) if next > offset => songs.next = next,
                    Some(_) => songs.failed = true,
                    None => songs.complete = true,
                }
            }
            Err(_) => songs.failed = true,
        }
        self.check_playlist_membership();
    }
}

#[cfg(all(test, feature = "demo"))]
mod tests {
    use super::*;
    use crate::{app::AppOptions, backend::Waker, paths::AppDirs, settings::Settings};

    #[test]
    fn playlist_membership_reads_pages_and_rejects_old_edits_and_accounts() {
        let root = std::env::temp_dir().join(format!("spotifast-membership-{}", std::process::id()));
        let mut app = App::new(&Waker::default(), AppDirs { config: root.join("config"), state: root.join("state"), cache: root.join("cache") }, Settings::default(), AppOptions { media_controls: false, restore_sign_in: false, tray: false, autostart: false });
        crate::demo::populate(&mut app);
        app.playlist_pages.clear();
        app.check_playlist_membership();
        assert_eq!(app.playlist_membership.playlists.values().filter(|songs| songs.pending.is_some()).count(), 2);
        let (id, (token, offset)) = app.playlist_membership.playlists.iter().find_map(|(id, songs)| songs.pending.map(|pending| (id.clone(), pending))).unwrap();
        assert_eq!(offset, 0);
        let item = PlayableItem::Track(crate::api::models::Track { uri: "spotify:track:member".into(), ..Default::default() });
        let row = PlaylistItem { item: Some(item.clone()), ..Default::default() };
        app.receive_playlist_membership(&id, 0, token, Ok(Page { items: vec![row.clone()], total: 100, limit: 50, offset: 0, next: Some("next".into()) }));
        assert_eq!(app.playlist_contains_song(&id, item.uri()), Some(true));
        assert_eq!(app.playlist_contains_song(&id, "spotify:track:absent"), None);
        let (next_token, offset) = app.playlist_membership.playlists[&id].pending.unwrap();
        assert_eq!(offset, 50);
        app.receive_playlist_membership(&id, offset, next_token, Ok(Page::whole(vec![])));
        assert_eq!(app.playlist_contains_song(&id, "spotify:track:absent"), Some(false));
        app.playlist_membership.invalidate(&id);
        app.playlist_membership.remove(&id, &[item.uri().to_string()]);
        app.receive_playlist_membership(&id, 0, token, Ok(Page::whole(vec![row])));
        assert_eq!(app.playlist_contains_song(&id, item.uri()), Some(false), "late reads cannot undo a removal");
        app.user.as_mut().unwrap().id = "different-account".into();
        app.sync_playlist_membership();
        assert_eq!(app.playlist_contains_song(&id, item.uri()), None);
        app.backend.shutdown();
    }
}
