//! Extension to the base installer with verification and installation of missing
//! Mojang versions, it also provides support for common arguments and fixes on legacy
//! versions.

pub(crate) mod serde;

use std::io::{self, BufReader, Write as _};
use std::path::{Path, PathBuf};
use std::collections::HashSet;
use std::fmt::Write as _;
use std::env;
use std::fs;

use chrono::{DateTime, FixedOffset};
use sha1::{Digest, Sha1};
use regex::Regex;
use uuid::Uuid;

use crate::base::{self, check_file_advanced, Game, HandlerInto as _, LibraryDownload, LoadedLibrary, VersionChannel, LIBRARIES_URL};
use crate::maven::Gav;
use crate::download;
use crate::msa;


/// Static URL to the version manifest provided by Mojang.
pub(crate) const DEFAULT_MANIFEST_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

/// An installer for supporting Mojang-provided versions. It provides support for various
/// standard arguments such as demo mode, window resolution and quick play, it also 
/// provides various fixes for known issues of old versions.
/// 
/// By default, this installer tries to handle every version that is not found in the
/// hierarchy, if a version with that name exists in the Mojang's versions manifest, then
/// it is fetched. This means that every missing version will try to fetch the manifest
/// (using the cached version if relevant).
/// 
/// Notes about various versions:
/// - 1.19.3 metadata adds no parameter to specify extract directory for LWJGL (version
///   3.3.1-build-7), therefore natives are extracted to 
///   '/tmp/lwjgl&lt;username&gt;/&lt;version&gt;'.
#[derive(Debug, Clone)]
pub struct Installer {
    /// The underlying base installer logic.
    base: base::Installer,
    /// Inner installer data, put in a sub struct to fix borrow issue.
    inner: InstallerInner,
}

/// Internal installer data.
#[derive(Debug, Clone)]
struct InstallerInner {
    version: Version,
    fetch_excludes: Vec<FetchExclude>,
    demo: bool,
    quick_play: Option<QuickPlay>,
    resolution: Option<(u16, u16)>,
    disable_multiplayer: bool,
    disable_chat: bool,
    auth_type: String,  // Empty to trigger default auth.
    auth_uuid: Uuid,
    auth_username: String,
    auth_token: String,
    auth_xuid: String,  // Apparently used for Minecraft Telemetry
    client_id: String,  // Apparently used for Minecraft Telemetry
    fix_legacy_quick_play: bool,
    fix_legacy_proxy: bool,
    fix_legacy_merge_sort: bool,
    fix_legacy_resolution: bool,
    fix_broken_authlib: bool,
    fix_lwjgl: Option<String>,
    manifest_url: Option<String>,
    manifest_prefix: Option<String>,
}

impl Installer {

    /// Create a new installer with default configuration, using defaults directories.
    /// 
    /// This Mojang installer has all fixes enabled except LWJGL and missing version 
    /// fetching is enabled.
    pub fn new(version: impl Into<Version>) -> Self {
        Self {
            base: base::Installer::new(String::new()),
            inner: InstallerInner {
                version: version.into(),
                fetch_excludes: Vec::new(),  // No exclude by default.
                demo: false,
                quick_play: None,
                resolution: None,
                disable_multiplayer: false,
                disable_chat: false,
                auth_type: String::new(),
                auth_uuid: Uuid::nil(),
                auth_username: String::new(),
                auth_token: String::new(),
                auth_xuid: String::new(),
                client_id: String::new(),
                fix_legacy_quick_play: true,
                fix_legacy_proxy: true,
                fix_legacy_merge_sort: true,
                fix_legacy_resolution: true,
                fix_broken_authlib: true,
                fix_lwjgl: None,
                manifest_url: None,
                manifest_prefix: None,
            }
        }
    }

    /// Same as [`Self::new`] but targets latest release version by default.
    pub fn new_with_release() -> Self {
        Self::new(Version::Release)
    }

    /// Get the underlying base installer.
    #[inline]
    pub fn base(&self) -> &base::Installer {
        &self.base
    }

    /// Get the underlying base installer through mutable reference.
    /// 
    /// *Note that the `version` property will be overwritten when installing.*
    #[inline]
    pub fn base_mut(&mut self) -> &mut base::Installer {
        &mut self.base
    }

    /// Get the mojang version to install.
    #[inline]
    pub fn version(&self) -> &Version {
        &self.inner.version
    }

    /// Set the mojang version to install.
    #[inline]
    pub fn set_version(&mut self, version: impl Into<Version>) -> &mut Self {
        self.inner.version = version.into();
        self
    }

    /// Return the list of filters 
    #[inline]
    pub fn fetch_excludes(&self) -> &[FetchExclude] {
        &self.inner.fetch_excludes
    }

    /// Clear all fetch exclude filters. See [`Self::fetch_excludes`] and 
    /// [`Self::add_fetch_exclude`]. **This is the default state when constructed.**
    pub fn clear_fetch_exclude(&mut self) -> &mut Self {
        self.inner.fetch_excludes.clear();
        self
    }

    /// Append the given filter to the fetch exclude list.
    pub fn add_fetch_exclude(&mut self, exclude: FetchExclude) -> &mut Self {
        self.inner.fetch_excludes.push(exclude);
        self
    }

    /// See [`Self::set_manifest_url`]
    #[inline]
    pub fn manifest_url(&self) -> Option<&str> {
        self.inner.manifest_url.as_deref()
    }

    /// Change the URL of the version manifest used to find missing versions and check
    /// if versions need to be updated. By default, this targets the public Mojang
    /// manifest, but this option can be used to specify a mirror, or other manifests.
    #[inline]
    pub fn set_manifest_url(&mut self, manifest_url: impl Into<String>) -> &mut Self {
        self.inner.manifest_url = Some(manifest_url.into());
        self
    }

    /// Use the default Mojang manifest, see [`Self::set_manifest_url`].
    #[inline]
    pub fn remove_manifest_url(&mut self) -> &mut Self {
        self.inner.manifest_url = None;
        self
    }

    /// See [`Self::set_manifest_prefix`]
    #[inline]
    pub fn manifest_prefix(&self) -> Option<&str> {
        self.inner.manifest_prefix.as_deref()
    }

    /// Change the prefix to set for all the versions of the configured version manifest.
    /// 
    /// For example with your custom manifest (see [`Self::set_manifest_url`]), you may
    /// not want to conflict with official Mojang versions (like `1.7.10`) if your version
    /// is slightly different.
    /// 
    /// In that case you are able to set the *manifest prefix* to something like `foo-`, 
    /// then you can only fetch versions starting with this prefix, such as `foo-1.7.10`.
    /// The version looked up in your custom manifest is still bare `1.7.10` but the 
    /// version is stored under `<versions_dir>/foo-1.7.10/`. The prefix is also added
    /// to the latest release and snapshot when using [`Version::Release`] or 
    /// [`Version::Snapshot`], but a [`Version::Name`] must include it.
    #[inline]
    pub fn set_manifest_prefix(&mut self, manifest_prefix: impl Into<String>) -> &mut Self {
        self.inner.manifest_prefix = Some(manifest_prefix.into());
        self
    }

    /// Don't prefix manifest versions, see [`Self::set_manifest_prefix`].
    #[inline]
    pub fn remove_manifest_prefix(&mut self) -> &mut Self {
        self.inner.manifest_prefix = None;
        self
    }

    /// Get if the demo mode is enabled for the game.
    #[inline]
    pub fn demo(&self) -> bool {
        self.inner.demo
    }

    /// Set if the demo mode should be enabled for the game.
    #[inline]
    pub fn set_demo(&mut self, demo: bool) -> &mut Self {
        self.inner.demo = demo;
        self
    }

    /// If enabled, get the Quick Play configuration when launching the game.
    #[inline]
    pub fn quick_play(&self) -> Option<&QuickPlay> {
        self.inner.quick_play.as_ref()
    }

    /// Enables Quick Play when launching the game, from 1.20 (23w14a).
    #[inline]
    pub fn set_quick_play(&mut self, quick_play: QuickPlay) -> &mut Self {
        self.inner.quick_play = Some(quick_play);
        self
    }

    /// Remove Quick Play when launching, this is the default.
    #[inline]
    pub fn remove_quick_play(&mut self) -> &mut Self {
        self.inner.quick_play = None;
        self
    }

    /// If enabled, get the initial game's window resolution.
    #[inline]
    pub fn resolution(&self) -> Option<(u16, u16)> {
        self.inner.resolution
    }

    /// Set an initial resolution for the game's window.
    #[inline]
    pub fn set_resolution(&mut self, width: u16, height: u16) -> &mut Self {
        self.inner.resolution = Some((width, height));
        self
    }

    /// Remove initial resolution for the game's window, this is the default.
    #[inline]
    pub fn remove_resolution(&mut self) -> &mut Self {
        self.inner.resolution = None;
        self
    }

    /// Get if multiplayer should be disabled when launching the game.
    #[inline]
    pub fn disable_multiplayer(&self) -> bool {
        self.inner.disable_multiplayer
    }

    /// Disable or not the multiplayer when launching the game.
    #[inline]
    pub fn set_disable_multiplayer(&mut self, disable_multiplayer: bool) -> &mut Self {
        self.inner.disable_multiplayer = disable_multiplayer;
        self
    }

    /// Get if the chat should be disabled when launching the game.
    #[inline]
    pub fn disable_chat(&self) -> bool {
        self.inner.disable_chat
    }

    /// Disable or not the chat when launching the game.
    #[inline]
    pub fn set_disable_chat(&mut self, disable_chat: bool) -> &mut Self {
        self.inner.disable_chat = disable_chat;
        self
    }

    /// Get the currently configured authentication UUID, may be nil (zero-filled) if not 
    /// configured.
    /// 
    /// Note that when installing, if this is nil then it will be defined using 
    /// [`Self::set_auth_offline_hostname`].
    #[inline]
    pub fn auth_uuid(&self) -> Uuid {
        self.inner.auth_uuid
    }

    /// Get the currently configured authentication UUID, may be empty if not configured.
    /// 
    /// Note that when installing, if this is nil then it will be defined using 
    /// [`Self::set_auth_offline_hostname`].
    #[inline]
    pub fn auth_username(&self) -> &str {
        &self.inner.auth_username
    }

    /// Internal function to reset to zero-length all online-related auth variables.
    fn reset_auth_online(&mut self) -> &mut Self {
        self.inner.auth_type = String::new();
        self.inner.auth_token = String::new();
        self.inner.auth_xuid = String::new();
        self
    }

    /// Use offline session with the given UUID and username, note that the username will
    /// be truncated 16 bytes at most (this function will panic if the truncation is not 
    /// on a valid UTF-8 character boundary).
    pub fn set_auth_offline(&mut self, uuid: Uuid, username: impl Into<String>) -> &mut Self {
        self.inner.auth_uuid = uuid;
        self.inner.auth_username = username.into();
        self.inner.auth_username.truncate(16);
        self.reset_auth_online()
    }

    /// Use offline session with the given UUID, the username is derived from the first
    /// 8 characters of the rendered UUID.
    pub fn set_auth_offline_uuid(&mut self, uuid: Uuid) -> &mut Self {
        self.inner.auth_uuid = uuid;
        self.inner.auth_username = uuid.to_string();
        self.inner.auth_username.truncate(8);
        self.reset_auth_online()
    }

    /// Use offline session with the given username (initially truncated to 16 chars), 
    /// the UUID is then derived from this username using the same derivation used by 
    /// most Mojang clients (versions to be defined), this produces a MD5 (v3) UUID 
    /// with `OfflinePlayer:{username}` as the hashed string.
    /// 
    /// The advantage of this method is to produce the same UUID as the one that will
    /// be produced by Mojang's authlib when connecting to an offline-mode multiplayer
    /// server.
    pub fn set_auth_offline_username(&mut self, username: impl Into<String>) -> &mut Self {
        
        self.inner.auth_username = username.into();
        self.inner.auth_username.truncate(16);

        let mut context = md5::Context::new();
        context.write_fmt(format_args!("OfflinePlayer:{}", self.inner.auth_username)).unwrap();
        
        self.inner.auth_uuid = uuid::Builder::from_bytes(context.finalize().0)
            .with_variant(uuid::Variant::RFC4122)
            .with_version(uuid::Version::Md5)
            .into_uuid();

        self.reset_auth_online()

    }

    /// Use offline session with the given username (initially truncated to 16 chars), 
    /// the UUID is then derived from this username using a PMC-specific derivation of 
    /// the username and the PMC namespace with SHA-1 (UUID v5).
    /// 
    /// Note that the produced UUID will not be used when playing on multiplayer servers
    /// (the server must also be in offline-mode), in this case the server gives you an
    /// arbitrary UUID that is not the one your game has been launched with. Most servers
    /// uses the UUID derivation embedded in Mojang's authlib, deriving the UUID from the
    /// username, if you want the UUID to be coherent with this derivation, you can use
    /// [`Self::set_auth_offline_username`] instead.
    pub fn set_auth_offline_username_legacy(&mut self, username: impl Into<String>) -> &mut Self {
        self.inner.auth_username = username.into();
        self.inner.auth_username.truncate(16);
        self.inner.auth_uuid = Uuid::new_v5(&base::UUID_NAMESPACE, self.inner.auth_username.as_bytes());
        self.reset_auth_online()
    }

    /// Use offline session with a deterministic UUID, derived from this machine's 
    /// hostname, the username is then derived from the UUID following the same logic
    /// as for [`Self::set_auth_offline_uuid`].
    /// 
    /// **This is the default UUID/username used if no auth is specified, so you don't
    /// need to call this function, except if you want to override previous auth.**
    pub fn set_auth_offline_hostname(&mut self) -> &mut Self {
        self.set_auth_offline_uuid(Uuid::new_v5(&base::UUID_NAMESPACE, gethostname::gethostname().as_encoded_bytes()))
    }

    /// Use online authentication with the given Microsoft Account.
    pub fn set_auth_msa(&mut self, account: &msa::Account) -> &mut Self {
        self.inner.auth_uuid = account.uuid();
        self.inner.auth_username = account.username().to_string();
        self.inner.auth_token = account.access_token().to_string();
        self.inner.auth_type = "msa".to_string();
        self.inner.auth_xuid = account.xuid().to_string();
        self
    }

    /// Get the client ID used for telemetry of the game, the default client id is empty
    /// and therefore the telemetry can't use it.
    #[inline]
    pub fn client_id(&self) -> &str {
        &self.inner.client_id
    }

    /// See [`Self::client_id`].
    #[inline]
    pub fn set_client_id(&mut self, client_id: impl Into<String>) -> &mut Self {
        self.inner.client_id = client_id.into();
        self
    }

    /// When starting versions older than 1.20 (23w14a) where Quick Play was not supported
    /// by the client, this fix tries to use legacy arguments instead, such as --server
    /// and --port, this is enabled by default.
    #[inline]
    pub fn fix_legacy_quick_play(&self) -> bool {
        self.inner.fix_legacy_quick_play
    }

    /// See [`Self::fix_legacy_quick_play`].
    #[inline]
    pub fn set_fix_legacy_quick_play(&mut self, fix: bool) -> &mut Self {
        self.inner.fix_legacy_quick_play = fix;
        self
    }

    /// When starting older alpha, beta and release up to 1.5, this allows legacy online
    /// resources such as skins to be properly requested. The implementation is currently 
    /// using `betacraft.uk` proxies, this is enabled by default.
    #[inline]
    pub fn fix_legacy_proxy(&self) -> bool {
        self.inner.fix_legacy_proxy
    }

    /// See [`Self::fix_legacy_proxy`].
    #[inline]
    pub fn set_fix_legacy_proxy(&mut self, fix: bool) -> &mut Self {
        self.inner.fix_legacy_proxy = fix;
        self
    }

    /// When starting older alpha and beta versions, this adds a JVM argument to use the
    /// legacy merge sort `java.util.Arrays.useLegacyMergeSort=true`, this is required on
    /// some old versions to avoid crashes, this is enabled by default.
    #[inline]
    pub fn fix_legacy_merge_sort(&self) -> bool {
        self.inner.fix_legacy_merge_sort
    }

    /// See [`Self::fix_legacy_merge_sort`].
    #[inline]
    pub fn set_fix_legacy_merge_sort(&mut self, fix: bool) -> &mut Self {
        self.inner.fix_legacy_merge_sort = fix;
        self
    }

    /// When starting older versions that don't support modern resolution arguments, this
    /// fix will add arguments to force resolution of the initial window, this is enabled 
    /// by default.
    #[inline]
    pub fn fix_legacy_resolution(&self) -> bool {
        self.inner.fix_legacy_resolution
    }

    /// See [`Self::fix_legacy_resolution`].
    #[inline]
    pub fn set_fix_legacy_resolution(&mut self, fix: bool) -> &mut Self {
        self.inner.fix_legacy_resolution = fix;
        self
    }

    /// Versions 1.16.4 and 1.16.5 uses authlib:2.1.28 which cause multiplayer button
    /// (and probably in-game chat) to be disabled, this can be fixed by switching to
    /// version 2.2.30 of authlib, this is enabled by default.
    #[inline]
    pub fn fix_broken_authlib(&self) -> bool {
        self.inner.fix_broken_authlib
    }

    /// See [`Self::fix_broken_authlib`].
    #[inline]
    pub fn set_fix_broken_authlib(&mut self, fix: bool) -> &mut Self {
        self.inner.fix_broken_authlib = fix;
        self
    }

    /// See [`Self::set_fix_lwjgl`].
    #[inline]
    pub fn fix_lwjgl(&self) -> Option<&str> {
        self.inner.fix_lwjgl.as_deref()
    }

    /// Changing the version of LWJGL, this support versions greater or equal to 3.2.3,
    /// and also provides ARM support when the LWJGL version supports it. It's not 
    /// guaranteed to work with every version of Minecraft, and downgrading LWJGL version
    /// is not recommended.
    /// 
    /// If the given version is less than 3.2.3 this will do nothing.
    #[inline]
    pub fn set_fix_lwjgl(&mut self, lwjgl_version: impl Into<String>) -> &mut Self {
        self.inner.fix_lwjgl = Some(lwjgl_version.into());
        self
    }
    
    /// Don't fix LWJGL version, see [`Self::set_fix_lwjgl`].
    #[inline]
    pub fn remove_fix_lwjgl(&mut self) -> &mut Self {
        self.inner.fix_lwjgl = None;
        self
    }

    /// Install the given Mojang version from its identifier. This also supports alias
    /// identifiers such as "release" and "snapshot" that will be resolved, note that
    /// these identifiers are just those presents in the "latest" mapping of the
    /// Mojang versions manifest. 
    /// 
    /// If the given version is not found in the manifest then it's silently ignored and
    /// the version metadata must already exists.
    #[inline]
    pub fn install(&mut self, mut handler: impl Handler) -> Result<Game> {
        self.install_dyn(&mut handler)
    }

    #[inline(never)]
    fn install_dyn(&mut self, handler: &mut dyn Handler) -> Result<Game> {
        
        // Apply default offline auth, derived from hostname.
        if self.inner.auth_uuid.is_nil() || self.inner.auth_username.is_empty() {
            self.set_auth_offline_hostname();
        }

        let &mut Self {
            ref mut base,
            ref inner,
        } = self;

        let manifest_url = self.inner.manifest_url.as_deref().unwrap_or(DEFAULT_MANIFEST_URL);
        let manifest = match self.inner.version {
            Version::Release | 
            Version::Snapshot => Some(Manifest::request_from_url(manifest_url, (&mut *handler).into_download())?),
            _ => None
        };

        // Versions from the manifest are bare, so the prefix is added, but a named
        // version is already the full name of the version to install.
        let manifest_prefix = self.inner.manifest_prefix.as_deref().unwrap_or_default();
        let version = match &self.inner.version {
            Version::Release => format!("{manifest_prefix}{}", manifest.as_ref().unwrap().latest_release_name()),
            Version::Snapshot => format!("{manifest_prefix}{}", manifest.as_ref().unwrap().latest_snapshot_name()),
            Version::Name(name) => name.clone(),
        };

        base.set_version(version);
        
        // Let the handler find the "leaf" version.
        let mut leaf_version = String::new();

        // Scoping the temporary internal handler.
        let mut game = {

            let mut handler = InternalHandler {
                inner: &mut *handler,
                installer: &inner,
                error: Ok(()),
                manifest,
                leaf_version: &mut leaf_version,
            };
    
            // Same as above, we are giving a &mut dyn ref to avoid huge monomorphization.
            let res = base.install(&mut handler);
            handler.error?;
            res?

        };

        // Apply auth parameters.
        game.replace_args(|arg| {
            Some(match arg {
                "auth_player_name" => inner.auth_username.clone(),
                "auth_uuid" => inner.auth_uuid.as_simple().to_string(),
                "auth_access_token" => inner.auth_token.clone(),
                "auth_xuid" => inner.auth_xuid.clone(),
                // Legacy parameter
                "auth_session" if !inner.auth_token.is_empty() => 
                    format!("token:{}:{}", inner.auth_token, inner.auth_uuid.as_simple()),
                "auth_session" => String::new(),
                "user_type" => inner.auth_type.clone(),
                "user_properties" => format!("{{}}"),
                "clientid" => inner.client_id.clone(),
                _ => return None
            })
        });

        // If Quick Play is enabled, we know that the feature has been enabled by the
        // handler, and if the feature is actually present (1.20 and after), if not
        // present we can try to use legacy arguments for supported quick play types.
        if let Some(quick_play) = &inner.quick_play {

            let quick_play_arg = match quick_play {
                QuickPlay::Path { .. } => "quickPlayPath",
                QuickPlay::Singleplayer { .. } => "quickPlaySingleplayer",
                QuickPlay::Multiplayer { .. } => "quickPlayMultiplayer",
                QuickPlay::Realms { .. } => "quickPlayRealms",
            };

            let mut quick_play_supported = false;
            game.replace_args(|arg| {
                if arg == quick_play_arg {
                    quick_play_supported = true;
                    Some(match quick_play {
                        QuickPlay::Path { path } => path.display().to_string(),
                        QuickPlay::Singleplayer { name } => name.clone(),
                        QuickPlay::Multiplayer { host, port } => format!("{host}:{port}"),
                        QuickPlay::Realms { id } => id.clone(),
                    })
                } else {
                    None
                }
            });

            if !quick_play_supported && inner.fix_legacy_quick_play {
                if let QuickPlay::Multiplayer { host, port } = quick_play {

                    game.game_args.extend([
                        "--server".to_string(), host.clone(),
                        "--port".to_string(), port.to_string(),
                    ]);

                    quick_play_supported = true;
                    handler.on_event(Event::FixedLegacyQuickPlay);

                }
            }

            if !quick_play_supported {
                handler.on_event(Event::WarnUnsupportedQuickPlay {  });
            }

        }

        if inner.fix_legacy_proxy {

            // Checking as bytes because it's ASCII and we simply matching.
            let proxy_port = match leaf_version.as_bytes() {
                [b'1', b'.', b'0' | b'1' | b'3' | b'4' | b'5'] |
                [b'1', b'.', b'2' | b'3' | b'4' | b'5', b'.', ..] |
                b"13w16a" | b"13w16b" => Some(11707),
                id if id.starts_with(b"a1.0.") => Some(80),
                id if id.starts_with(b"a1.1.") => Some(11702),
                id if id.starts_with(b"a1.") => Some(11705),
                id if id.starts_with(b"b1.") => Some(11705),
                _ => None,
            };

            if let Some(proxy_port) = proxy_port {
                game.jvm_args.push(format!("-Dhttp.proxyHost=betacraft.uk"));
                game.jvm_args.push(format!("-Dhttp.proxyPort={proxy_port}"));
                handler.on_event(Event::FixedLegacyProxy { host: "betacraft.uk", port: proxy_port });
            }

        }

        if inner.fix_legacy_merge_sort && (leaf_version.starts_with("a1.") || leaf_version.starts_with("b1.")) {
            game.jvm_args.push("-Djava.util.Arrays.useLegacyMergeSort=true".to_string());
            handler.on_event(Event::FixedLegacyMergeSort);
        }

        if let Some((width, height)) = inner.resolution {

            let mut resolution_supported = false;
            game.replace_args(|arg| {
                let repl = match arg {
                    "resolution_width" => width.to_string(),
                    "resolution_height" => height.to_string(),
                    _ => return None
                };
                resolution_supported = true;
                Some(repl)
            });

            if !resolution_supported && inner.fix_legacy_resolution {

                game.game_args.extend([
                    "--width".to_string(), width.to_string(),
                    "--height".to_string(), height.to_string(),
                ]);

                resolution_supported = true;
                handler.on_event(Event::FixedLegacyResolution);

            }

            if !resolution_supported {
                handler.on_event(Event::WarnUnsupportedResolution);
            }

        }

        if inner.disable_multiplayer {
            game.game_args.push("--disableMultiplayer".to_string());
        }

        if inner.disable_chat {
            game.game_args.push("--disableChat".to_string());
        }

        Ok(game)

    }

}

/// Events happening when installing.
#[derive(Debug)]
#[non_exhaustive]
pub enum Event<'a> {
    /// Forwarding a base event.
    Base(base::Event<'a>),
    /// When the given version is being loaded but the file has an invalid size,
    /// SHA-1, or any other invalidating reason, it has been removed in order to 
    /// download an up-to-date version.
    InvalidatedVersion { version: &'a str },
    /// The required version metadata is missing and so will be fetched.
    FetchVersion { version: &'a str },
    /// The version has been fetched.
    FetchedVersion { version: &'a str },
    /// Quick play has been fixed.
    FixedLegacyQuickPlay,
    /// Legacy proxy has been defined to fix legacy versions.
    FixedLegacyProxy { host: &'a str, port: u16 },
    /// Legacy merge sort has been fixed.
    FixedLegacyMergeSort,
    /// Legacy resolution arguments have been added.
    FixedLegacyResolution,
    /// Notification of a fix of authlib:2.1.28 has happened.
    FixedBrokenAuthlib,
    /// A quick play mode is requested by is not supported by this version, or the fix
    /// has been disabled. This is just a warning.
    WarnUnsupportedQuickPlay,
    /// A specific initial window resolution has been requested but it's not supported
    /// by the current version and the fix is disabled. This is just a warning.
    WarnUnsupportedResolution,
}

/// A handle for watching an installation.
pub trait Handler {
    /// Handle a single event.
    fn on_event(&mut self, event: Event);
}

// Mutable implementation.
impl<H: Handler + ?Sized> Handler for &mut H {
    #[inline]
    fn on_event(&mut self, event: Event) {
        (**self).on_event(event)
    }
}

impl Handler for () {
    fn on_event(&mut self, event: Event) {
        let _ = event;
    }
}

/// Internal adapter trait for using it like other handlers.
pub(crate) trait HandlerInto: Handler + Sized {
    
    #[inline]
    fn into_base(self) -> impl base::Handler {
        pub(crate) struct Adapter<H: Handler>(pub H);
        impl<H: Handler> base::Handler for Adapter<H> {
            fn on_event(&mut self, event: base::Event) {
                self.0.on_event(Event::Base(event));
            }
        }
        Adapter(self)
    }

    #[inline]
    fn into_download(self) -> impl download::Handler {
        self.into_base().into_download()
    }

}

impl<H: Handler> HandlerInto for H {}

/// The Mojang installer could not proceed to the installation of a version.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// Error from the base installer.
    #[error("base: {0}")]
    Base(#[source] base::Error),
    /// The LWJGL fix is enabled with a version that is not supported, maybe because
    /// it is too old (< 3.2.3) or because of your system not being supported.
    #[error("lwjgl fix not found: {version}")]
    LwjglFixNotFound {
        version: String,
    },
}

impl<T: Into<base::Error>> From<T> for Error {
    fn from(value: T) -> Self {
        Error::Base(value.into())
    }
}

/// Type alias for a result with the Mojang error type.
pub type Result<T> = std::result::Result<T, Error>;

/// The version to install.
#[derive(Debug, Clone)]
pub enum Version {
    /// Install the latest Mojang release.
    Release,
    /// Install the latest Mojang snapshot.
    Snapshot,
    /// Install this specific game version, if not a Mojang-provided version, it should
    /// be already installed in the versions directory.
    Name(String),
}

/// An impl so that we can give string-like objects to the builder.
impl<T: Into<String>> From<T> for Version {
    fn from(value: T) -> Self {
        Self::Name(value.into())
    }
}

/// This represent the optional Quick Play mode when launching the game. This is usually 
/// not supported on versions older than 1.20 (23w14a), however a fix exists for 
/// supporting multiplayer Quick Play on older versions, other modes are unsupported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuickPlay {
    /// Launch the game and follow instruction for Quick Play in the given path, relative
    /// to the working directory.
    Path {
        path: PathBuf,
    },
    /// Launch the game and directly join the world given its name.
    Singleplayer {
        name: String,
    },
    /// Launch the game and directly join the specified server address.
    Multiplayer {
        host: String,
        port: u16,
    },
    /// Launch the game and directly join the realm given its id.
    Realms {
        id: String,
    },
}

/// The different kind of patterns for filtering which versions are fetched or not.
#[derive(Debug, Clone)]
pub enum FetchExclude {
    /// Exclude all versions from being fetched from Mojang's manifest. It overrides
    /// all other excludes.
    All,
    /// Exclude a specific version name.
    Exact(String),
    /// Exclude a version's name that matches the given regex.
    Regex(Regex),
}

/// A handle to the Mojang versions manifest.
#[derive(Debug)]
pub struct Manifest {
    inner: Box<serde::MojangManifest>,
}

impl Manifest {

    /// Request the default Mojang versions' manifest.
    /// 
    /// Note that this takes a download handler since such file is quite huge and tracking
    /// the download could make sense!
    pub fn request(handler: impl download::Handler) -> Result<Self> {
        Self::request_from_url(DEFAULT_MANIFEST_URL, handler)
    }

    /// Request a Mojang version's manifest at the given URL.
    /// 
    /// Note that this takes a download handler since such file is quite huge and tracking
    /// the download could make sense!
    pub fn request_from_url(url: &str, mut handler: impl download::Handler) -> Result<Self> {
        Self::request_dyn(url, &mut handler)
    }

    fn request_dyn(url: &str, handler: &mut dyn download::Handler) -> Result<Self> {

        let mut entry = download::single_cached(url)
            .set_keep_open()
            .download(handler)?;

        let reader = BufReader::new(entry.take_handle().unwrap());
        let mut deserializer = serde_json::Deserializer::from_reader(reader);
        let manifest = serde_path_to_error::deserialize::<_, Box<serde::MojangManifest>>(&mut deserializer)
            .map_err(|e| base::Error::new_json_file(e, entry.file()))?;

        Ok(Self { inner: manifest })

    }

    /// Iterator over all versions in the manifest.
    /// 
    /// This method currently returns an abstract iterator because the API is not 
    /// stabilized yet.
    pub fn iter(&self) -> impl Iterator<Item = ManifestVersion<'_>> + use<'_> {
        self.inner.versions.iter()
            .map(ManifestVersion)
    }

    /// Return the latest release version name.
    #[inline]
    pub fn latest_release_name(&self) -> &str {
        &self.inner.latest.release
    }

    /// Return the latest snapshot version name.
    #[inline]
    pub fn latest_snapshot_name(&self) -> &str {
        &self.inner.latest.snapshot
    }

    /// Return the latest version name for the given "latest" key.
    pub fn latest_name(&self, key: &str) -> Option<&str> {
        match key {
            "release" => Some(self.latest_release_name()),
            "snapshot" => Some(self.latest_snapshot_name()),
            _ => self.inner.latest.extra.get(key).map(String::as_str),
        }
    }

    /// Find the index of a version given its name.
    pub fn find_index_of_name(&self, name: &str) -> Option<usize> {
        self.inner.versions.iter().position(|v| v.id == name)
    }

    /// Get a version from its index within the manifest.
    pub fn find_by_index(&self, index: usize) -> Option<ManifestVersion<'_>> {
        self.inner.versions.get(index).map(ManifestVersion)
    }

    /// Get a handle to a version information from its name.
    pub fn find_by_name(&self, name: &str) -> Option<ManifestVersion<'_>> {
        self.inner.versions.iter()
            .find(|v| v.id == name)
            .map(ManifestVersion)
    }

}

/// A handle to a version in the Mojang versions manifest.
#[derive(Debug)]
pub struct ManifestVersion<'a>(&'a serde::MojangManifestVersion);

impl<'a> ManifestVersion<'a> {

    /// The name of this version.
    /// 
    /// See [`base::LoadedVersion::name`] for more information on the naming.
    pub fn name(&self) -> &'a str {
        &self.0.id
    }

    /// The release channel of this version.
    pub fn channel(&self) -> VersionChannel {
        self.0.r#type.parse().unwrap()
    }

    /// The last update time for this version.
    pub fn time(&self) -> &'a DateTime<FixedOffset> {
        &self.0.time
    }

    /// The release time for this version. 
    pub fn release_time(&self) -> &'a DateTime<FixedOffset> {
        &self.0.release_time
    }

    /// Return the download URL to this version metadata.
    pub fn url(&self) -> &'a str {
        &self.0.download.url
    }

    /// Return the expected size of this version metadata, if any.
    pub fn size(&self) -> Option<u32> {
        self.0.download.size
    }

    /// Return the expected SHA-1 of this version metadata, if any.
    pub fn sha1(&self) -> Option<&'a [u8; 20]> {
        self.0.download.sha1.as_deref()
    }

}

// ========================== //
// Following code is internal //
// ========================== //

/// Internal handler given to the base installer.
struct InternalHandler<'a> {
    /// Inner handler.
    inner: &'a mut dyn Handler,
    /// Back-reference to the installer to know its configuration.
    installer: &'a InstallerInner,
    /// If there is an error in the handler.
    error: Result<()>,
    /// If fetching is enabled, then this contains the manifest to use.
    manifest: Option<Manifest>,
    /// Id of the "leaf" version, the last version without inherited version.
    leaf_version: &'a mut String,
}

impl<'a> base::Handler for InternalHandler<'a> {
    
    fn on_event(&mut self, mut event: base::Event) {
        
        let ret = match event {
            base::Event::FilterFeatures { 
                ref mut features,
            } => self.filter_features(*features),
            base::Event::LoadedHierarchy {
                hierarchy,
            } => self.loaded_hierarchy(hierarchy),
            base::Event::LoadVersion { 
                version, 
                file,
            } => self.load_version(version, file),
            base::Event::NeedVersion { 
                version, 
                file, 
                ref mut retry,
            } => match self.need_version(version, file) {
                Ok(true) => {
                    **retry = true;
                    Ok(())
                }
                Ok(false) => Ok(()),
                Err(e) => Err(e),
            }
            base::Event::FilterLibraries { 
                ref mut libraries,
            } => self.filter_libraries(*libraries),
            _ => Ok(())
        };
        
        if let Err(e) = ret {
            self.error = Err(e);
            return;
        }

        self.inner.on_event(Event::Base(event));

    }

}

impl InternalHandler<'_> {

    fn filter_features(&mut self, features: &mut HashSet<String>) -> Result<()> {
        
        if self.installer.demo {
            features.insert("is_demo_user".to_string());
        }

        if self.installer.resolution.is_some() {
            features.insert("has_custom_resolution".to_string());
        }

        if let Some(quick_play) = &self.installer.quick_play {
            features.insert(match quick_play {
                QuickPlay::Path { .. } => "has_quick_plays_support",
                QuickPlay::Singleplayer { .. } => "is_quick_play_singleplayer",
                QuickPlay::Multiplayer { .. } => "is_quick_play_multiplayer",
                QuickPlay::Realms { .. } => "is_quick_play_realms",
            }.to_string());
        }

        Ok(())

    }

    fn loaded_hierarchy(&mut self, hierarchy: &[base::LoadedVersion]) -> Result<()> {
        // The manifest prefix is stripped so that legacy fixes can match the bare name.
        let leaf_version = hierarchy.last().unwrap().name();
        let manifest_prefix = self.installer.manifest_prefix.as_deref().unwrap_or_default();
        *self.leaf_version = leaf_version.strip_prefix(manifest_prefix).unwrap_or(leaf_version).to_string();
        Ok(())
    }

    fn load_version(&mut self, version: &str, file: &Path) -> Result<()> {

        // If any pattern matches, return Ok.
        for pattern in &self.installer.fetch_excludes {
            match pattern {
                FetchExclude::All => 
                    return Ok(()),
                FetchExclude::Exact(name) if name == version => 
                    return Ok(()),
                FetchExclude::Regex(regex) if regex.is_match(version) => 
                    return Ok(()),
                _ => (),
            }
        }

        // If a manifest prefix is defined, we can only handle versions that starts with
        // this prefix, this is some kind of fetch filter like above, but this in 
        // particular is used to strip the version and derive to the manifest's version.
        let manifest_prefix = self.installer.manifest_prefix.as_deref().unwrap_or_default();
        let Some(manifest_version) = version.strip_prefix(manifest_prefix) else {
            return Ok(());
        };

        // Only ensure that the manifest is loaded after checking fetch exclude.
        let manifest_url = self.installer.manifest_url.as_deref().unwrap_or(DEFAULT_MANIFEST_URL);
        let manifest = match self.manifest {
            Some(ref manifest) => manifest,
            None => self.manifest.insert(Manifest::request_from_url(manifest_url, (&mut *self.inner).into_download())?)
        };

        let Some(manifest_version) = manifest.find_by_name(manifest_version) else {
            return Ok(());
        };

        // If the manifest version is found. It means that when this version was 
        // installed, a .xsha1 file was created and contains the XOR of the original 
        // expected sha1 and the sha1 of the file after its "id" field was patched. 
        // This allows knowing the sha1 expected for that file even after it's patched 
        // by only storing one XOR.
        //
        // EXAMPLE: This file can therefore be manually created by anyone who wants to 
        // include its own modifications and keep them.
        let mut sha1 = manifest_version.sha1().copied();

        if let Some(sha1) = &mut sha1 {
            let xsha1_file = file.with_added_extension("xsha1");
            match fs::read_to_string(&xsha1_file) {
                Ok(raw) => {
                    if let Some(xsha1) = crate::serde::parse_hex_bytes::<20>(&raw) {
                        *sha1 = xor_bytes(sha1, &xsha1);
                    }
                }
                Err(e) if let io::ErrorKind::NotFound = e.kind() => {
                    // Not having xsha1 is purely valid, but at the risk of forcing 
                    // re-fetch if the SHA-1 is not matching!
                }
                Err(e) => return Err(base::Error::new_io_file(e, &xsha1_file).into())
            };
        }

        // The size of a patched file differs from the manifest's one, so it's not checked.
        let size = manifest_version.size().filter(|_| manifest_prefix.is_empty());

        if !check_file_advanced(file, size, sha1.as_ref(), true)? {
            
            fs::remove_file(file)
                .map_err(|e| base::Error::new_io_file(e, file))?;
            
            self.inner.on_event(Event::InvalidatedVersion { version });
        
        }

        Ok(())

    }

    fn need_version(&mut self, version: &str, file: &Path) -> Result<bool> {

        let Some(manifest) = self.manifest.as_ref() else {
            return Ok(false);
        };
        
        let manifest_prefix = self.installer.manifest_prefix.as_deref().unwrap_or_default();
        let Some(manifest_version) = version.strip_prefix(manifest_prefix) else {
            return Ok(false);
        };
        
        let Some(manifest_version) = manifest.find_by_name(manifest_version) else {
            return Ok(false);
        };
        
        self.inner.on_event(Event::FetchVersion { version });
        
        // When we have a prefix, download to .tmp so that the version file is only
        // written once the .xsha1 file is written, so this remains valid if interrupted.
        let entry_file =
            if !manifest_prefix.is_empty() {
                &file.with_added_extension("tmp")
            } else {
                file
            };

        let mut entry = download::single(manifest_version.url(), entry_file)
            .set_expected_size(manifest_version.size())
            .set_expected_sha1(manifest_version.sha1().copied())
            .set_keep_open()
            .download((&mut *self.inner).into_download())?;

        // With a manifest prefix, the "id" field is patched to match the prefixed name,
        // and the XOR of the original and patched sha1 is stored in the .xsha1 file, so
        // that the file can still be checked against the manifest's sha1 later.
        if !manifest_prefix.is_empty() {

            let patched = {

                let reader = BufReader::new(entry.take_handle().unwrap());

                let mut deserializer = serde_json::Deserializer::from_reader(reader);
                let mut metadata = serde_path_to_error::deserialize::<_, serde_json::Map<String, serde_json::Value>>(&mut deserializer)
                    .map_err(|e| base::Error::new_json_file(e, entry_file))?;

                metadata.insert("id".to_string(), serde_json::Value::String(version.to_string()));

                let mut patched = Vec::new();
                let mut serializer = serde_json::Serializer::new(&mut patched);
                serde_path_to_error::serialize(&metadata, &mut serializer)
                    .map_err(|e| base::Error::new_json_file(e, entry_file))?;

                patched

            };

            // Ignore error while deleting the tmp file, not critical...
            let _ = fs::remove_file(entry_file);

            let patched_sha1: [u8; 20] = Sha1::digest(&patched).into();
            
            let mut xsha1_raw = String::with_capacity(40);
            for byte in xor_bytes(entry.sha1(), &patched_sha1) {
                write!(xsha1_raw, "{byte:02x}").unwrap();
            }

            let xsha1_file = file.with_added_extension("xsha1");
            fs::write(&xsha1_file, xsha1_raw)
                .map_err(|e| base::Error::new_io_file(e, &xsha1_file))?;

            // Once xsha1 is written, write the version file and remove .tmp one...
            fs::write(file, &patched)
                .map_err(|e| base::Error::new_io_file(e, file))?;

        }

        self.inner.on_event(Event::FetchedVersion { version });

        Ok(true)

    }

    fn filter_libraries(&mut self, libraries: &mut Vec<LoadedLibrary>) -> Result<()> {
        
        if self.installer.fix_broken_authlib {
            self.apply_fix_broken_authlib(&mut *libraries)?;
        }

        if let Some(lwjgl_version) = self.installer.fix_lwjgl.as_deref() {
            self.apply_fix_lwjgl(&mut *libraries, lwjgl_version)?;
        }

        Ok(())

    }

    fn apply_fix_broken_authlib(&mut self, libraries: &mut Vec<LoadedLibrary>) -> Result<()> {

        // Unwrap because we don't exceed length limit for sure.
        let target_gav = Gav::new("com.mojang", "authlib", "2.1.28", None, None).unwrap();
        let pos = libraries.iter().position(|lib| lib.name == target_gav);
    
        if let Some(pos) = pos {

            libraries[pos].path = None;  // Ensure that the path is recomputed.
            libraries[pos].name = libraries[pos].name.with_version("2.2.30").unwrap();
            libraries[pos].download = Some(LibraryDownload {
                url: format!("{LIBRARIES_URL}{}", libraries[pos].name.url()),
                size: Some(87497),
                sha1: Some([0xd6, 0xe6, 0x77, 0x19, 0x9a, 0xa6, 0xb1, 0x9c, 0x4a, 0x9a, 0x2e, 0x72, 0x50, 0x34, 0x14, 0x9e, 0xb3, 0xe7, 0x46, 0xf8]),
            });

            self.inner.on_event(Event::FixedBrokenAuthlib);

        }

        Ok(())
    
    }
    
    fn apply_fix_lwjgl(&mut self, libraries: &mut Vec<LoadedLibrary>, version: &str) -> Result<()> {
    
        let Some(("", minor_patch)) = version.split_once("3.") else {
            return Err(Error::LwjglFixNotFound { 
                version: version.to_string(),
            });
        };

        if minor_patch != "2.3" {
            
            let (minor, _) = minor_patch.split_once('.').unwrap_or((minor_patch, ""));
            let Ok(minor) = minor.parse::<u32>() else {
                return Err(Error::LwjglFixNotFound { 
                    version: version.to_string(),
                });
            };

            if minor < 3 {
                return Err(Error::LwjglFixNotFound { 
                    version: version.to_string(),
                });
            }

        }
        
        let classifier = match (env::consts::OS, env::consts::ARCH) {
            ("windows", "x86") => "natives-windows-x86",
            ("windows", "x86_64") => "natives-windows",
            ("windows", "aarch64") if version != "3.2.3" => "natives-windows-arm64",
            ("linux", "x86" | "x86_64") => "natives-linux",
            ("linux", "arm") => "natives-linux-arm32",
            ("linux", "aarch64") => "natives-linux-arm64",
            ("macos", "x86_64") => "natives-macos",
            ("macos", "aarch64") if version != "3.2.3" => "natives-macos-arm64",
            _ => return Err(Error::LwjglFixNotFound { 
                version: version.to_string(),
            })
        };
    
        // Contains to-be-expected unique LWJGL libraries, with the classifier.
        let mut lwjgl_libs = Vec::new();
    
        // Start by not retaining libraries with classifiers (natives).
        libraries.retain_mut(|lib| {
            if let ("org.lwjgl", "jar") = (lib.name.group(), lib.name.extension()) {
                if lib.name.classifier().is_none() {
                    if let Some(new_name) = lib.name.with_version(version) 
                    && let Some(new_classifier_name) = new_name.with_classifier(Some(classifier)) {
                        lib.path = None;
                        lib.download = None;  // Will be updated afterward.
                        lib.name = new_name;
                        lwjgl_libs.push(new_classifier_name);
                        true
                    } else {
                        // Do not retain if we got an error create the new version, it 
                        // might be too long and we don't want to unwrap.
                        false
                    }
                } else {
                    // Libraries with classifiers are not retained.
                    false
                }
            } else {
                true
            }
        });
    
        // Now we add the classifiers for each LWJGL lib.
        libraries.extend(lwjgl_libs.into_iter().map(|gav| {
            LoadedLibrary {
                name: gav,
                path: None,
                download: None, // Will be set in the loop just after.
                natives: false,
            }
        }));
    
        // Finally we update the download source.
        for lib in libraries {
            if let ("org.lwjgl", "jar") = (lib.name.group(), lib.name.extension()) {
                let url = format!("https://repo1.maven.org/maven2/{}", lib.name.url());
                lib.download = Some(LibraryDownload { url, size: None, sha1: None });
            }
        }

        Ok(())
    
    }

}

fn xor_bytes<const N: usize>(a: &[u8; N], b: &[u8; N]) -> [u8; N] {
    let mut res = [0u8; N];
    for i in 0..N {
        res[i] = a[i] ^ b[i];
    }
    res
}
