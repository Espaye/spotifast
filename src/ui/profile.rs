//! A listener's profile and the two lists behind its counts.
use crate::api::models::pick_image;
use crate::app::App;
use crate::i18n::{gettext, ngettext};
use crate::model::{Action, Page, ProfilePage};
use crate::session_reads::{ProfileEntry, ProfileView};
use crate::theme::{self, Icon};

use super::collection::{Hero, HeroImages, hero};
use super::widgets::{self, CardCover};

pub(super) fn show(app: &mut App, ui: &mut egui::Ui, id: &str, route: &Page) {
    let page = std::mem::take(&mut app.profile);
    let own = app.user_id() == Some(id);
    let view = (page.id == id && app.local_ready)
        .then(|| page.view.get())
        .flatten();
    let account = app.user.as_ref().filter(|u| u.id == id);
    let friend = app.friends.get().and_then(|friends| {
        friends
            .iter()
            .find(|f| f.user_uri.strip_prefix("spotify:user:") == Some(id))
    });
    let name = view
        .and_then(|v| v.user.name.as_deref())
        .or_else(|| account.map(|u| u.name()))
        .or_else(|| app.user_names.get(id).and_then(Option::as_deref))
        .or_else(|| friend.map(|f| f.user_name.as_str()))
        .unwrap_or(id)
        .to_owned();
    let image = view
        .and_then(|v| v.user.image_url.as_deref())
        .or_else(|| account.and_then(|u| pick_image(&u.images, 320)))
        .or_else(|| app.user_images.get(id).and_then(Option::as_deref))
        .or_else(|| friend.and_then(|f| f.image_url.as_deref()))
        .map(str::to_owned);
    let palette = app.palette;
    let locale = app.locale;
    match route {
        Page::User(_) => {
            let mut byline = Vec::new();
            if let Some(view) = view {
                if let Some(count) = view.playlist_count {
                    byline.push((
                        ngettext(
                            locale,
                            "{count} public playlist",
                            "{count} public playlists",
                            count.try_into().unwrap_or(u32::MAX),
                        )
                        .replace("{count}", &crate::util::format_count(count)),
                        None,
                    ));
                }
                if !view.followers.is_empty() {
                    let count = view.followers_count.unwrap_or(view.followers.len() as u64);
                    byline.push((
                        ngettext(
                            locale,
                            "{count} follower",
                            "{count} followers",
                            count.try_into().unwrap_or(u32::MAX),
                        )
                        .replace("{count}", &crate::util::format_count(count)),
                        Some(Page::Followers(id.into())),
                    ));
                }
                if !view.following.is_empty() {
                    let count = view.following_count.unwrap_or(view.following.len() as u64);
                    byline.push((
                        gettext(locale, "{count} following")
                            .replace("{count}", &crate::util::format_count(count)),
                        Some(Page::Following(id.into())),
                    ));
                }
            }
            hero(
                app,
                ui,
                Hero {
                    images: HeroImages {
                        image: image.as_deref(),
                        enlarged: image.as_deref(),
                        thumbnail: image.as_deref(),
                        ..Default::default()
                    },
                    liked: false,
                    kind: gettext(locale, "Profile"),
                    title: &name,
                    description: None,
                    byline,
                    byline_tooltip: None,
                    avatar: None,
                    round: true,
                },
            );
            if own {
                let more = theme::icon_button(
                    ui,
                    Icon::Ellipsis,
                    26.0,
                    palette.secondary,
                    palette.text,
                    &gettext(locale, "More"),
                );
                egui::Popup::menu(&more)
                    .frame(widgets::menu_frame(&palette))
                    .show(|ui| {
                        if widgets::menu_item(
                            ui,
                            &palette,
                            None,
                            &gettext(locale, "Edit profile in Spotify"),
                        ) {
                            let id = percent_encoding::utf8_percent_encode(
                                id,
                                percent_encoding::NON_ALPHANUMERIC,
                            );
                            app.actions.push(Action::OpenUrl(format!(
                                "https://open.spotify.com/user/{id}"
                            )));
                        }
                    });
                if page.id == id {
                    top_shelves(app, ui, &page);
                }
            }
            if let Some(entries) = view.and_then(|v| v.playlists.as_ref()) {
                shelf(
                    app,
                    ui,
                    "profile-playlists",
                    &gettext(locale, "Public playlists"),
                    entries,
                );
            } else if own {
                let entries = app
                    .library
                    .playlists
                    .get()
                    .into_iter()
                    .flatten()
                    .filter(|p| p.public == Some(true) && p.owner.id.as_deref() == Some(id))
                    .take(50)
                    .map(|p| ProfileEntry {
                        page: Page::Playlist(p.id.clone()),
                        name: p.name.clone(),
                        image: pick_image(&p.images, 200).map(str::to_owned),
                    })
                    .collect::<Vec<_>>();
                shelf(
                    app,
                    ui,
                    "profile-playlists",
                    &gettext(locale, "Public playlists"),
                    &entries,
                );
            }
            if let Some(view) = view {
                shelf(
                    app,
                    ui,
                    "profile-followers",
                    &gettext(locale, "Followers"),
                    &view.followers[..view.followers.len().min(20)],
                );
                shelf(
                    app,
                    ui,
                    "profile-following",
                    &gettext(locale, "Following"),
                    &view.following[..view.following.len().min(20)],
                );
            }
        }
        Page::Followers(_) | Page::Following(_) => people(app, ui, route, view, page.filter, &name),
        _ => {}
    }
    if !app.local_ready && (!own || !matches!(route, Page::User(_))) {
        theme::subtle(
            ui,
            &palette,
            &gettext(locale, "Connect Spotify to see this profile."),
        );
        super::settings::spotify_access_control(app, ui);
    } else if page.id == id && page.view.is_loading() {
        widgets::loading_row(ui, &palette, locale);
    }
    app.profile = page;
}

fn shelf(app: &mut App, ui: &mut egui::Ui, id: &str, title: &str, entries: &[ProfileEntry]) {
    if entries.is_empty() {
        return;
    }
    let palette = app.palette;
    widgets::shelf(ui, &palette, id, title, |ui| {
        for entry in entries {
            let circular = matches!(entry.page, Page::Artist(_) | Page::User(_));
            let label = match entry.page {
                Page::Artist(_) => gettext(app.locale, "Artist"),
                Page::User(_) => gettext(app.locale, "Profile"),
                _ => gettext(app.locale, "Playlist"),
            };
            if widgets::card(
                ui,
                app,
                entry.image.as_deref(),
                &entry.name,
                &label,
                CardCover {
                    circular,
                    ..Default::default()
                },
            )
            .clicked
            {
                app.actions.push(Action::Open(entry.page.clone()));
            }
        }
    });
}

fn top_shelves(app: &mut App, ui: &mut egui::Ui, page: &ProfilePage) {
    let locale = app.locale;
    let palette = app.palette;
    if let Some(artists) = page.top_artists.get().filter(|items| !items.is_empty()) {
        theme::section_title(ui, &palette, &gettext(locale, "Top artists this month"));
        theme::subtle(ui, &palette, &gettext(locale, "Only visible to you"));
        widgets::shelf_cards(ui, "profile-top-artists", |ui| {
            for a in artists {
                if widgets::card(
                    ui,
                    app,
                    pick_image(&a.images, 200),
                    &a.name,
                    &gettext(locale, "Artist"),
                    CardCover {
                        circular: true,
                        ..Default::default()
                    },
                )
                .clicked
                {
                    app.actions.push(Action::Open(Page::Artist(a.id.clone())));
                }
            }
        });
    }
    if let Some(tracks) = page.top_tracks.get().filter(|items| !items.is_empty()) {
        theme::section_title(ui, &palette, &gettext(locale, "Top tracks this month"));
        theme::subtle(ui, &palette, &gettext(locale, "Only visible to you"));
        widgets::shelf_cards(ui, "profile-top-tracks", |ui| {
            for (index, t) in tracks.iter().enumerate() {
                let response = widgets::card(
                    ui,
                    app,
                    t.image(200),
                    &t.name,
                    &t.artist_names(),
                    CardCover::square(false),
                );
                if response.play || response.clicked {
                    app.actions.push(Action::PlayUris {
                        uris: tracks.iter().map(|t| t.uri.clone()).collect(),
                        index,
                    });
                }
            }
        });
    }
}

fn people(
    app: &mut App,
    ui: &mut egui::Ui,
    route: &Page,
    view: Option<&ProfileView>,
    filter: Option<bool>,
    name: &str,
) {
    let palette = app.palette;
    let locale = app.locale;
    let followers = matches!(route, Page::Followers(_));
    let title = if followers {
        gettext(locale, "Followers")
    } else {
        gettext(locale, "Following")
    };
    theme::section_title(ui, &palette, &title);
    theme::subtle(ui, &palette, name);
    let Some(view) = view else { return };
    let entries = if followers {
        &view.followers
    } else {
        &view.following
    };
    if entries.is_empty() {
        theme::subtle(ui, &palette, &gettext(locale, "No public list to show."));
        return;
    }
    let artist = |e: &ProfileEntry| matches!(e.page, Page::Artist(_));
    let mixed = entries.iter().any(artist) && entries.iter().any(|e| !artist(e));
    if mixed
        && let Some(value) = widgets::chips(
            ui,
            &palette,
            &[
                (None, &gettext(locale, "All")),
                (Some(true), &gettext(locale, "Artists")),
                (Some(false), &gettext(locale, "Profiles")),
            ],
            filter,
        )
    {
        app.actions.push(Action::FilterPeople(value));
    }
    for entry in entries
        .iter()
        .filter(|e| !mixed || filter.is_none_or(|want| want == artist(e)))
    {
        let row = ui.horizontal(|ui| {
            widgets::cover(ui, &palette, entry.image.as_deref(), 40.0, 20.0, Icon::User);
            ui.vertical(|ui| {
                theme::text(ui, &entry.name, theme::semibold(14.0), palette.text);
                theme::subtle(
                    ui,
                    &palette,
                    &if artist(entry) {
                        gettext(locale, "Artist")
                    } else {
                        gettext(locale, "Profile")
                    },
                );
            });
        });
        if ui
            .interact(
                row.response.rect,
                row.response.id.with("open"),
                egui::Sense::click(),
            )
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .clicked()
        {
            app.actions.push(Action::Open(entry.page.clone()));
        }
    }
}
