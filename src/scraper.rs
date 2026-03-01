use anyhow::{Context, Result};
use fantoccini::cookies::Cookie as FantocciniCookie;
use fantoccini::{Client, ClientBuilder};
use rusqlite::Connection;
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// Detected social media platform
#[derive(Debug, Clone, PartialEq)]
pub enum Platform {
    LinkedIn,
    Facebook,
    Twitter,
    Unknown,
}

impl Platform {
    pub fn as_str(&self) -> &str {
        match self {
            Platform::LinkedIn => "linkedin",
            Platform::Facebook => "facebook",
            Platform::Twitter => "twitter",
            Platform::Unknown => "unknown",
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            Platform::LinkedIn => "LinkedIn",
            Platform::Facebook => "Facebook",
            Platform::Twitter => "Twitter/X",
            Platform::Unknown => "Unknown",
        }
    }
}

/// A cookie read from Firefox's cookies.sqlite
#[derive(Debug, Clone)]
struct FirefoxCookie {
    host: String,
    name: String,
    value: String,
    path: String,
    expiry: i64,
    is_secure: bool,
    is_http_only: bool,
    same_site: i32,
}

/// Detect the social media platform from a URL
pub fn detect_platform(url: &str) -> Platform {
    let url_lower = url.to_lowercase();
    if url_lower.contains("linkedin.com") {
        Platform::LinkedIn
    } else if url_lower.contains("facebook.com") || url_lower.contains("fb.com") {
        Platform::Facebook
    } else if url_lower.contains("twitter.com") || url_lower.contains("x.com") {
        Platform::Twitter
    } else {
        Platform::Unknown
    }
}

/// Auto-detect the user's Firefox profile directory.
/// Returns the first valid default profile path found, or None.
pub fn find_firefox_profile(config_override: Option<&Path>) -> Option<PathBuf> {
    // If the user specified a path in config, use that
    if let Some(path) = config_override {
        if path.exists() {
            info!("Using configured Firefox profile path: {}", path.display());
            return Some(path.to_path_buf());
        } else {
            warn!(
                "Configured Firefox profile path does not exist: {}",
                path.display()
            );
        }
    }

    // Auto-detect based on OS
    let home = std::env::var("HOME").ok().map(PathBuf::from);

    #[cfg(target_os = "linux")]
    let firefox_dir = home.as_ref().map(|h| h.join(".mozilla/firefox"));

    #[cfg(target_os = "macos")]
    let firefox_dir = home
        .as_ref()
        .map(|h| h.join("Library/Application Support/Firefox/Profiles"));

    #[cfg(target_os = "windows")]
    let firefox_dir = std::env::var("APPDATA")
        .ok()
        .map(PathBuf::from)
        .map(|d| d.join("Mozilla/Firefox/Profiles"));

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    let firefox_dir: Option<PathBuf> = None;

    if let Some(ref ff_dir) = firefox_dir {
        // Try to parse profiles.ini to find the default profile
        let profiles_ini = ff_dir.join("profiles.ini");
        if profiles_ini.exists() {
            if let Ok(content) = std::fs::read_to_string(&profiles_ini) {
                let mut current_path: Option<String> = None;
                let mut current_is_relative: bool = true;
                let mut default_profile: Option<PathBuf> = None;

                for line in content.lines() {
                    let line = line.trim();
                    if line.starts_with("[Profile") {
                        current_path = None;
                        current_is_relative = true;
                    } else if let Some(path) = line.strip_prefix("Path=") {
                        current_path = Some(path.to_string());
                    } else if line == "IsRelative=0" {
                        current_is_relative = false;
                    } else if line.starts_with("Default=1")
                        || line.starts_with("Name=default-release")
                    {
                        if let Some(ref path) = current_path {
                            let profile_path = if current_is_relative {
                                ff_dir.join(path)
                            } else {
                                PathBuf::from(path)
                            };
                            if profile_path.exists() {
                                info!(
                                    "Auto-detected Firefox profile at: {}",
                                    profile_path.display()
                                );
                                return Some(profile_path);
                            }
                        }
                    }

                    if let Some(ref path) = current_path {
                        if line.starts_with("Default=1") {
                            let profile_path = if current_is_relative {
                                ff_dir.join(path)
                            } else {
                                PathBuf::from(path)
                            };
                            if profile_path.exists() {
                                default_profile = Some(profile_path);
                            }
                        }
                    }
                }

                if let Some(dp) = default_profile {
                    info!(
                        "Auto-detected Firefox default profile at: {}",
                        dp.display()
                    );
                    return Some(dp);
                }
            }
        }

        // Fallback: look for a directory matching *.default-release or *.default
        if ff_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(ff_dir) {
                let mut candidates: Vec<PathBuf> = entries
                    .filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .filter(|p| p.is_dir())
                    .collect();
                candidates.sort_by(|a, b| {
                    let a_name = a.file_name().unwrap_or_default().to_string_lossy();
                    let b_name = b.file_name().unwrap_or_default().to_string_lossy();
                    let a_prio = if a_name.ends_with(".default-release") {
                        0
                    } else if a_name.ends_with(".default") {
                        1
                    } else {
                        2
                    };
                    let b_prio = if b_name.ends_with(".default-release") {
                        0
                    } else if b_name.ends_with(".default") {
                        1
                    } else {
                        2
                    };
                    a_prio.cmp(&b_prio)
                });
                for candidate in candidates {
                    let name = candidate
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    if name.ends_with(".default-release") || name.ends_with(".default") {
                        info!(
                            "Auto-detected Firefox profile at: {}",
                            candidate.display()
                        );
                        return Some(candidate);
                    }
                }
            }
        }
    }

    warn!("No Firefox profile directory found. Scraping will run without authentication.");
    None
}

/// Read cookies from Firefox's cookies.sqlite for a specific domain.
/// Copies the database to a temp file first so it works even while Firefox is running.
fn read_firefox_cookies(profile_path: &Path, domain: &str) -> Result<Vec<FirefoxCookie>> {
    let cookies_db = profile_path.join("cookies.sqlite");
    if !cookies_db.exists() {
        anyhow::bail!(
            "Firefox cookies.sqlite not found at: {}",
            cookies_db.display()
        );
    }

    // Copy to a temp file since Firefox may have the DB locked
    let temp_dir = std::env::temp_dir();
    let temp_db = temp_dir.join(format!("profiler_cookies_{}.sqlite", std::process::id()));
    std::fs::copy(&cookies_db, &temp_db).context("Failed to copy cookies.sqlite to temp location")?;

    // Also copy the WAL file if it exists (needed for recent cookie changes)
    let wal_file = profile_path.join("cookies.sqlite-wal");
    let temp_wal = temp_dir.join(format!("profiler_cookies_{}.sqlite-wal", std::process::id()));
    if wal_file.exists() {
        let _ = std::fs::copy(&wal_file, &temp_wal);
    }

    let conn = Connection::open(&temp_db).context("Failed to open cookies database")?;

    // Query cookies matching the domain (including subdomains via leading dot)
    let mut stmt = conn
        .prepare(
            "SELECT host, name, value, path, expiry, isSecure, isHttpOnly, sameSite \
             FROM moz_cookies \
             WHERE host LIKE ?1 OR host LIKE ?2",
        )
        .context("Failed to prepare cookies query")?;

    let domain_pattern = format!("%{}", domain);
    let dot_domain_pattern = format!(".{}", domain);

    let cookies = stmt
        .query_map([&domain_pattern, &dot_domain_pattern], |row| {
            Ok(FirefoxCookie {
                host: row.get(0)?,
                name: row.get(1)?,
                value: row.get(2)?,
                path: row.get(3)?,
                expiry: row.get(4)?,
                is_secure: row.get::<_, i32>(5)? != 0,
                is_http_only: row.get::<_, i32>(6)? != 0,
                same_site: row.get(7)?,
            })
        })
        .context("Failed to query cookies")?
        .filter_map(|r| r.ok())
        .collect::<Vec<_>>();

    // Cleanup temp files
    let _ = std::fs::remove_file(&temp_db);
    let _ = std::fs::remove_file(&temp_wal);

    info!(
        "Read {} cookies for domain '{}' from Firefox profile",
        cookies.len(),
        domain
    );

    Ok(cookies)
}

/// Get the cookie domains needed for a given platform
fn get_cookie_domains(platform: &Platform) -> Vec<&'static str> {
    match platform {
        Platform::Facebook => vec!["facebook.com", "fb.com"],
        Platform::LinkedIn => vec!["linkedin.com"],
        Platform::Twitter => vec!["twitter.com", "x.com"],
        Platform::Unknown => vec![],
    }
}

/// Get the base URL for a platform (needed to set the domain before injecting cookies)
fn get_platform_base_url(platform: &Platform) -> &'static str {
    match platform {
        Platform::Facebook => "https://www.facebook.com",
        Platform::LinkedIn => "https://www.linkedin.com",
        Platform::Twitter => "https://x.com",
        Platform::Unknown => "about:blank",
    }
}

/// Inject Firefox cookies into the geckodriver session
async fn inject_cookies(
    client: &Client,
    profile_path: &Path,
    platform: &Platform,
) -> Result<usize> {
    let domains = get_cookie_domains(platform);
    let mut total_injected = 0;

    // Navigate to the platform's base URL first (cookies can only be set for the current domain)
    let base_url = get_platform_base_url(platform);
    if base_url != "about:blank" {
        info!("Navigating to {} to set cookie domain...", base_url);
        client
            .goto(base_url)
            .await
            .context("Failed to navigate to base URL for cookie injection")?;
        // Brief wait for the page to start loading
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }

    for domain in domains {
        match read_firefox_cookies(profile_path, domain) {
            Ok(cookies) => {
                for cookie in &cookies {
                    // Build a cookie for fantoccini/WebDriver
                    // Strip leading dot from domain for WebDriver compatibility
                    let domain_val = if cookie.host.starts_with('.') {
                        cookie.host[1..].to_string()
                    } else {
                        cookie.host.clone()
                    };

                    let built_cookie = FantocciniCookie::build(
                        FantocciniCookie::new(cookie.name.clone(), cookie.value.clone()),
                    )
                    .path(cookie.path.clone())
                    .secure(cookie.is_secure)
                    .http_only(cookie.is_http_only)
                    .domain(domain_val)
                    .build();

                    match client.add_cookie(built_cookie).await {
                        Ok(()) => {
                            total_injected += 1;
                        }
                        Err(e) => {
                            debug!(
                                "Failed to inject cookie '{}' for {}: {}",
                                cookie.name, cookie.host, e
                            );
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Failed to read cookies for domain '{}': {}", domain, e);
            }
        }
    }

    info!(
        "Successfully injected {} cookies for {}",
        total_injected,
        platform.display_name()
    );
    Ok(total_injected)
}

/// Managed geckodriver process
struct GeckoDriver {
    process: Child,
    port: u16,
}

impl GeckoDriver {
    /// Start geckodriver on an available port
    fn start() -> Result<Self> {
        let port = {
            let listener = std::net::TcpListener::bind("127.0.0.1:0")
                .context("Failed to find available port for geckodriver")?;
            listener.local_addr()?.port()
        };

        info!("Starting geckodriver on port {}", port);
        let process = Command::new("geckodriver")
            .arg("--port")
            .arg(port.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context(
                "Failed to start geckodriver. Is it installed?\n\
                 Try: sudo apt install firefox-geckodriver\n\
                 Or download from: https://github.com/mozilla/geckodriver/releases",
            )?;

        // Give geckodriver a moment to start
        std::thread::sleep(Duration::from_millis(500));

        Ok(GeckoDriver { process, port })
    }
}

impl Drop for GeckoDriver {
    fn drop(&mut self) {
        debug!("Shutting down geckodriver (pid {})", self.process.id());
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

/// Scrape text content from a social media profile URL using headless Firefox.
///
/// If `firefox_profile_path` is provided (from config or auto-detected),
/// cookies from the user's Firefox profile will be injected for authenticated access.
/// This works even while Firefox is running (reads cookies from a copy of the database).
pub async fn scrape_url(url: &str, firefox_profile_path: Option<PathBuf>) -> Result<String> {
    let platform = detect_platform(url);
    info!("Scraping {} URL: {}", platform.display_name(), url);

    // Try with cookies first, fall back to anonymous
    let result = if let Some(ref profile_path) = firefox_profile_path {
        info!(
            "Attempting authenticated scrape using Firefox cookies from: {}",
            profile_path.display()
        );
        match scrape_with_cookies(url, &platform, Some(profile_path)).await {
            Ok(text) => Ok(text),
            Err(e) => {
                warn!(
                    "Authenticated scrape failed: {}. Retrying without cookies...",
                    e
                );
                scrape_with_cookies(url, &platform, None).await
            }
        }
    } else {
        scrape_with_cookies(url, &platform, None).await
    };

    result
}

/// Inner scraping function that optionally injects Firefox cookies.
async fn scrape_with_cookies(
    url: &str,
    platform: &Platform,
    profile_path: Option<&PathBuf>,
) -> Result<String> {
    // Start geckodriver
    let geckodriver = GeckoDriver::start()?;

    // Build Firefox capabilities
    let mut firefox_opts = Map::new();
    let args = vec![
        json!("--headless"),
        json!("--width=1920"),
        json!("--height=1080"),
    ];
    firefox_opts.insert("args".to_string(), json!(args));

    // Set a realistic user agent via preferences
    let mut prefs = Map::new();
    prefs.insert(
        "general.useragent.override".to_string(),
        json!("Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0"),
    );
    prefs.insert("dom.webdriver.enabled".to_string(), json!(false));
    prefs.insert("useAutomationExtension".to_string(), json!(false));
    prefs.insert("dom.webnotifications.enabled".to_string(), json!(false));
    prefs.insert("geo.enabled".to_string(), json!(false));
    firefox_opts.insert("prefs".to_string(), json!(prefs));

    let mut capabilities = Map::new();
    capabilities.insert("moz:firefoxOptions".to_string(), json!(firefox_opts));

    let webdriver_url = format!("http://localhost:{}", geckodriver.port);

    info!("Launching headless Firefox (anonymous session, cookies will be injected)...");

    let client = ClientBuilder::native()
        .capabilities(capabilities)
        .connect(&webdriver_url)
        .await
        .context(
            "Failed to connect to geckodriver/Firefox. Is Firefox installed?\n\
             Try: sudo apt install firefox geckodriver",
        )?;

    // Inject cookies from Firefox profile if available
    if let Some(profile) = profile_path {
        match inject_cookies(&client, profile, platform).await {
            Ok(count) => {
                info!("Injected {} cookies from Firefox profile", count);
            }
            Err(e) => {
                warn!("Failed to inject cookies: {}. Proceeding without authentication.", e);
            }
        }
    }

    // Run the scraping logic
    let result = do_scrape(&client, url, platform, profile_path.is_some()).await;

    // Always close the browser session
    if let Err(e) = client.close().await {
        warn!("Failed to close Firefox session: {}", e);
    }

    drop(geckodriver);

    result
}

/// Perform the actual scraping after browser is connected and cookies are set
async fn do_scrape(
    client: &Client,
    url: &str,
    platform: &Platform,
    has_profile: bool,
) -> Result<String> {
    // Navigate to the actual target URL
    info!("Navigating to {}", url);
    client
        .goto(url)
        .await
        .context("Failed to navigate to URL")?;

    // Wait for page to load and dynamic content to render
    info!("Waiting for dynamic content to render...");
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Diagnostic: log what page we actually loaded
    let page_title = execute_js_string(client, "return document.title || '';").await;
    let page_url = execute_js_string(client, "return window.location.href || '';").await;
    let body_length_str = execute_js_string(
        client,
        "return (document.body && document.body.innerHTML.length || 0).toString();",
    )
    .await;
    let body_length = body_length_str.parse::<usize>().unwrap_or(0);

    info!(
        "Page loaded - title: '{}', url: '{}', body_length: {}",
        page_title, page_url, body_length
    );

    // Check if we hit a login page
    if page_url.contains("/login")
        || page_url.contains("/signin")
        || page_url.contains("checkpoint")
        || page_url.contains("/uas/")
        || page_url.contains("accounts.google.com")
    {
        if has_profile {
            warn!("Redirected to login page despite using cookies. Session may have expired.");
        } else {
            warn!("Redirected to login page. Use authenticated mode for better results.");
        }
    }

    if body_length < 100 {
        warn!(
            "Page body is very short ({} chars). Site may be blocking headless Firefox.",
            body_length
        );
    }

    // Scroll down to load more posts (social media uses infinite scroll)
    let scroll_count = match platform {
        Platform::LinkedIn => 8,
        Platform::Facebook => 8,
        Platform::Twitter => 10,
        Platform::Unknown => 5,
    };

    info!(
        "Scrolling page {} times to load dynamic content...",
        scroll_count
    );
    for i in 0..scroll_count {
        let _ = execute_js_string(
            client,
            "window.scrollBy(0, window.innerHeight); return '';",
        )
        .await;
        tokio::time::sleep(Duration::from_millis(1200)).await;
        debug!("Scroll iteration {}/{} complete", i + 1, scroll_count);
    }

    // Scroll back to top before extraction
    let _ = execute_js_string(client, "window.scrollTo(0, 0); return '';").await;
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Log selector diagnostics
    log_selector_diagnostics(client, platform).await;

    // Extract text based on platform
    let extracted = match platform {
        Platform::LinkedIn => extract_linkedin_text(client).await,
        Platform::Facebook => extract_facebook_text(client).await,
        Platform::Twitter => extract_twitter_text(client).await,
        Platform::Unknown => extract_generic_text(client).await,
    };

    match extracted {
        Ok(text) if text.trim().is_empty() => {
            warn!("Platform-specific extraction returned empty, trying generic extraction");
            let generic = extract_generic_text(client).await?;
            if generic.trim().is_empty() {
                let dump = execute_js_string(
                    client,
                    "return (document.body ? document.body.innerText.substring(0, 5000) : '(no body)');",
                )
                .await;
                error!(
                    "All extraction methods returned empty. Page dump ({} chars): {}",
                    dump.len(),
                    &dump[..dump.len().min(2000)]
                );

                anyhow::bail!(
                    "Could not extract any text from {}.\n\n\
                     Page title: '{}'\n\
                     Page URL: '{}'\n\
                     Body size: {} chars\n\n\
                     The page may require authentication or the content may not be publicly accessible.\n\n\
                     Tips:\n\
                     • Make sure you're logged into {} in Firefox\n\
                     • Check that the URL is correct and the profile is public\n\
                     • Some sites aggressively block automated access",
                    url,
                    page_title,
                    page_url,
                    body_length,
                    platform.display_name()
                );
            }
            info!(
                "Generic extraction recovered {} characters",
                generic.len()
            );
            Ok(generic)
        }
        Ok(text) => {
            info!(
                "Successfully extracted {} characters from {}",
                text.len(),
                platform.display_name()
            );
            Ok(text)
        }
        Err(e) => {
            warn!(
                "Platform-specific extraction failed: {}, trying generic",
                e
            );
            extract_generic_text(client).await
        }
    }
}

/// Execute a JavaScript snippet and return the result as a String.
async fn execute_js_string(client: &Client, script: &str) -> String {
    match client.execute(script, vec![]).await {
        Ok(value) => match value {
            Value::String(s) => s,
            Value::Null => String::new(),
            other => {
                let s = other.to_string();
                s.trim_matches('"').to_string()
            }
        },
        Err(e) => {
            warn!(
                "JS execution failed: {} for script: {}",
                e,
                &script[..script.len().min(100)]
            );
            String::new()
        }
    }
}

/// Log how many elements match platform-specific selectors (for debugging)
async fn log_selector_diagnostics(client: &Client, platform: &Platform) {
    let selectors: Vec<(&str, &str)> = match platform {
        Platform::LinkedIn => vec![
            (
                "h1 (name)",
                "return document.querySelectorAll('h1').length.toString();",
            ),
            (
                "about section",
                "return document.querySelectorAll('#about').length.toString();",
            ),
            (
                "feed posts",
                "return document.querySelectorAll('.feed-shared-update-v2__description, .feed-shared-text, .update-components-text').length.toString();",
            ),
            (
                "experience items",
                "return document.querySelectorAll('.pvs-list__paged-list-item').length.toString();",
            ),
        ],
        Platform::Facebook => vec![
            (
                "h1 (name)",
                "return document.querySelectorAll('h1').length.toString();",
            ),
            (
                "posts",
                "return document.querySelectorAll('div[data-ad-preview=\"message\"], div[dir=\"auto\"]').length.toString();",
            ),
        ],
        Platform::Twitter => vec![
            (
                "user name",
                "return document.querySelectorAll('[data-testid=\"UserName\"]').length.toString();",
            ),
            (
                "tweets",
                "return document.querySelectorAll('[data-testid=\"tweetText\"]').length.toString();",
            ),
        ],
        Platform::Unknown => vec![],
    };

    for (label, js) in selectors {
        let count = execute_js_string(client, js).await;
        info!(
            "Selector diagnostic [{}]: {} = {}",
            platform.display_name(),
            label,
            count
        );
    }
}

/// Extract text from LinkedIn profile/posts
async fn extract_linkedin_text(client: &Client) -> Result<String> {
    let js = r#"
    return (function() {
        var texts = [];
        var nameEl = document.querySelector('.text-heading-xlarge, .pv-text-details--left-panel h1, h1.inline, h1');
        if (nameEl) texts.push('Name: ' + nameEl.innerText.trim());
        var headlineEl = document.querySelector('.text-body-medium, .pv-text-details--left-panel .text-body-medium');
        if (headlineEl) texts.push('Headline: ' + headlineEl.innerText.trim());
        var aboutSelectors = [
            '#about ~ div .inline-show-more-text',
            '#about + div .inline-show-more-text',
            '#about + .display-flex .inline-show-more-text',
            '.pv-about-section .inline-show-more-text',
            'section.pv-about-section div',
            '#about + .display-flex .inline-show-more-text--is-collapsed',
            '#about ~ .display-flex span[aria-hidden="true"]',
            'section:has(#about) .display-flex span[aria-hidden="true"]',
            'section:has(#about) .inline-show-more-text span'
        ];
        for (var i = 0; i < aboutSelectors.length; i++) {
            var about = document.querySelector(aboutSelectors[i]);
            if (about && about.innerText.trim().length > 20) {
                texts.push('\nAbout:\n' + about.innerText.trim());
                break;
            }
        }
        var experienceItems = document.querySelectorAll(
            '.pvs-list__paged-list-item, .pv-entity__summary-info, ' +
            'section[id*="experience"] li .display-flex, section:has(#experience) li, ' +
            '#experience ~ .pvs-list__outer-container li'
        );
        if (experienceItems.length > 0) {
            texts.push('\n--- Experience ---');
            var seen = {};
            for (var i = 0; i < experienceItems.length; i++) {
                var text = experienceItems[i].innerText.trim();
                if (text.length > 20 && !seen[text]) { seen[text] = true; texts.push(text); }
            }
        }
        var postSelectors = [
            '.feed-shared-update-v2__description', '.feed-shared-text',
            '.feed-shared-inline-show-more-text', '.update-components-text',
            '.break-words span[dir="ltr"]',
            'div.feed-shared-update-v2 .feed-shared-text__text-view',
            'span.break-words span',
            '.feed-shared-update-v2 .feed-shared-text-view span',
            '.occludable-update .update-components-text span'
        ];
        var posts = document.querySelectorAll(postSelectors.join(', '));
        if (posts.length > 0) {
            texts.push('\n--- Posts & Activity ---');
            var seen = {};
            for (var i = 0; i < posts.length; i++) {
                var t = posts[i].innerText.trim();
                if (t.length > 10 && !seen[t]) { seen[t] = true; texts.push(t); }
            }
        }
        var comments = document.querySelectorAll(
            '.comments-comment-item__main-content, .feed-shared-main-content, .comments-comment-texteditor .ql-editor'
        );
        if (comments.length > 0) {
            texts.push('\n--- Comments ---');
            var seen = {};
            for (var i = 0; i < comments.length; i++) {
                var t = comments[i].innerText.trim();
                if (t.length > 10 && !seen[t]) { seen[t] = true; texts.push(t); }
            }
        }
        var skills = document.querySelectorAll(
            'section[id*="skills"] .pvs-list__paged-list-item span[aria-hidden="true"], ' +
            'section:has(#skills) .pvs-list__paged-list-item span[aria-hidden="true"]'
        );
        if (skills.length > 0) {
            texts.push('\n--- Skills ---');
            var skillList = [];
            for (var i = 0; i < skills.length; i++) {
                var t = skills[i].innerText.trim();
                if (t.length > 1 && skillList.indexOf(t) === -1) skillList.push(t);
            }
            texts.push(skillList.join(', '));
        }
        return texts.join('\n\n');
    })();
    "#;
    let text = execute_js_string(client, js).await;
    if text.starts_with("ERROR:") {
        anyhow::bail!("LinkedIn extraction JS error: {}", text);
    }
    Ok(text)
}

/// Extract text from Facebook profile/posts
async fn extract_facebook_text(client: &Client) -> Result<String> {
    let js = r#"
    return (function() {
        var texts = [];
        var nameEl = document.querySelector('h1');
        if (nameEl) texts.push('Name: ' + nameEl.innerText.trim());
        var introSelectors = [
            '[data-pagelet="ProfileTilesFeed_0"] span', '.bi6gxh9e span',
            '[data-pagelet="ProfileTilesFeed"] li span', 'div[data-pagelet="intro_card"] span',
            '[data-pagelet*="ProfileTiles"] span', 'div[data-pagelet*="intro"] span'
        ];
        var introTexts = {};
        for (var s = 0; s < introSelectors.length; s++) {
            var els = document.querySelectorAll(introSelectors[s]);
            for (var i = 0; i < els.length; i++) {
                var t = els[i].innerText.trim();
                if (t.length > 5) introTexts[t] = true;
            }
        }
        var introKeys = Object.keys(introTexts);
        if (introKeys.length > 0) {
            texts.push('\n--- Bio/Intro ---');
            for (var i = 0; i < introKeys.length; i++) texts.push(introKeys[i]);
        }
        var postSelectors = [
            'div[data-ad-preview="message"]', 'div[data-ad-comet-preview="message"]',
            '.userContent', '[data-testid="post_message"] span',
            'div[dir="auto"][style*="text-align"]', 'div[data-ad-preview="message"] span',
            'div[class*="x1iorvi4"] div[dir="auto"]', 'div.x1n2onr6 div[dir="auto"]',
            'div[data-ad-comet-preview="message"] div[dir="auto"]',
            'div[role="article"] div[dir="auto"]', 'div[role="article"] span[dir="auto"]'
        ];
        var postTexts = {};
        for (var s = 0; s < postSelectors.length; s++) {
            var els = document.querySelectorAll(postSelectors[s]);
            for (var i = 0; i < els.length; i++) {
                var t = els[i].innerText.trim();
                if (t.length > 15) postTexts[t] = true;
            }
        }
        var postKeys = Object.keys(postTexts);
        if (postKeys.length > 0) {
            texts.push('\n--- Posts ---');
            for (var i = 0; i < postKeys.length; i++) texts.push(postKeys[i]);
        }
        var commentSelectors = [
            '.UFICommentBody', '[role="article"] span[dir="auto"]',
            'div[aria-label*="comment"] span[dir="auto"]', 'ul[role="list"] div[dir="auto"]'
        ];
        var commentTexts = {};
        for (var s = 0; s < commentSelectors.length; s++) {
            var els = document.querySelectorAll(commentSelectors[s]);
            for (var i = 0; i < els.length; i++) {
                var t = els[i].innerText.trim();
                if (t.length > 10) commentTexts[t] = true;
            }
        }
        var commentKeys = Object.keys(commentTexts);
        if (commentKeys.length > 0) {
            texts.push('\n--- Comments ---');
            for (var i = 0; i < commentKeys.length; i++) texts.push(commentKeys[i]);
        }
        return texts.join('\n\n');
    })();
    "#;
    let text = execute_js_string(client, js).await;
    if text.starts_with("ERROR:") {
        anyhow::bail!("Facebook extraction JS error: {}", text);
    }
    Ok(text)
}

/// Extract text from Twitter/X profile/posts
async fn extract_twitter_text(client: &Client) -> Result<String> {
    let js = r#"
    return (function() {
        var texts = [];
        var displayName = document.querySelector('[data-testid="UserName"] span, [data-testid="UserName"] div span');
        if (displayName) texts.push('Name: ' + displayName.innerText.trim());
        var bio = document.querySelector('[data-testid="UserDescription"]');
        if (bio) texts.push('Bio: ' + bio.innerText.trim());
        var location = document.querySelector('[data-testid="UserLocation"]');
        if (location) texts.push('Location: ' + location.innerText.trim());
        var tweets = document.querySelectorAll('[data-testid="tweetText"]');
        if (tweets.length > 0) {
            texts.push('\n--- Tweets ---');
            var seen = {};
            for (var i = 0; i < tweets.length; i++) {
                var t = tweets[i].innerText.trim();
                if (t.length > 0 && !seen[t]) { seen[t] = true; texts.push(t); }
            }
        }
        return texts.join('\n\n');
    })();
    "#;
    let text = execute_js_string(client, js).await;
    if text.starts_with("ERROR:") {
        anyhow::bail!("Twitter extraction JS error: {}", text);
    }
    Ok(text)
}

/// Generic text extraction fallback - grabs visible text from the page body
async fn extract_generic_text(client: &Client) -> Result<String> {
    let js = r#"
    return (function() {
        var walker = document.createTreeWalker(
            document.body, NodeFilter.SHOW_TEXT,
            { acceptNode: function(node) {
                var parent = node.parentElement;
                if (!parent) return NodeFilter.FILTER_REJECT;
                var tag = parent.tagName.toLowerCase();
                if (['script','style','noscript','meta','link','svg','path'].indexOf(tag) !== -1)
                    return NodeFilter.FILTER_REJECT;
                try {
                    var style = window.getComputedStyle(parent);
                    if (style.display === 'none' || style.visibility === 'hidden')
                        return NodeFilter.FILTER_REJECT;
                } catch(e) {}
                if (node.textContent.trim().length < 3) return NodeFilter.FILTER_REJECT;
                return NodeFilter.FILTER_ACCEPT;
            }}
        );
        var texts = []; var node;
        while (node = walker.nextNode()) {
            var t = node.textContent.trim();
            if (t.length >= 3) texts.push(t);
        }
        var seen = {}; var unique = [];
        for (var i = 0; i < texts.length; i++) {
            if (!seen[texts[i]]) { seen[texts[i]] = true; unique.push(texts[i]); }
        }
        return unique.join('\n');
    })();
    "#;
    let text = execute_js_string(client, js).await;
    if text.starts_with("ERROR:") {
        anyhow::bail!("Generic extraction JS error: {}", text);
    }
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_platform() {
        assert_eq!(
            detect_platform("https://www.linkedin.com/in/someone"),
            Platform::LinkedIn
        );
        assert_eq!(
            detect_platform("https://linkedin.com/in/someone"),
            Platform::LinkedIn
        );
        assert_eq!(
            detect_platform("https://www.facebook.com/someone"),
            Platform::Facebook
        );
        assert_eq!(detect_platform("https://fb.com/someone"), Platform::Facebook);
        assert_eq!(
            detect_platform("https://twitter.com/someone"),
            Platform::Twitter
        );
        assert_eq!(detect_platform("https://x.com/someone"), Platform::Twitter);
        assert_eq!(detect_platform("https://example.com"), Platform::Unknown);
    }

    #[test]
    fn test_find_firefox_profile_with_invalid_override() {
        // An invalid override path should NOT be returned as the result.
        // The function may still auto-detect a real profile on this machine,
        // so we only verify the bogus path wasn't used.
        let bogus = Path::new("/nonexistent/path");
        let result = find_firefox_profile(Some(bogus));
        if let Some(ref path) = result {
            assert_ne!(path.as_path(), bogus, "Invalid override should be skipped");
        }
        // result may be Some (auto-detected) or None (no Firefox installed) — both ok
    }
}
