use tauri::Url;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Destination {
    Internal,
    Blob,
    External,
    Blocked,
}

fn host_matches(host: &str, domain: &str) -> bool {
    host == domain || host.ends_with(&format!(".{domain}"))
}

fn path_matches(path: &str, base: &str) -> bool {
    path == base || path.starts_with(&format!("{base}/"))
}

pub(crate) fn destination(url: &Url) -> Destination {
    match url.scheme() {
        "blob" => return Destination::Blob,
        "mailto" | "tel" => return Destination::External,
        "http" | "https" => {}
        // Do not dispatch arbitrary protocols to Windows from a remote page.
        _ => return Destination::Blocked,
    }

    let host = url.host_str().unwrap_or("");
    let path = url.path();
    let facebook_app_host = matches!(
        host,
        "facebook.com" | "www.facebook.com" | "m.facebook.com" | "web.facebook.com"
    );
    let facebook_app_path = [
        "/messages",
        "/login",
        "/login.php",
        "/checkpoint",
        "/two_step_verification",
        "/recover",
        "/identify",
        "/confirmemail.php",
        "/device-based",
        "/dialog/oauth",
    ]
    .iter()
    .any(|base| path_matches(path, base));

    // This is a navigation policy, not a network/subresource filter. Images,
    // videos, APIs and scripts can still load from their normal CDN domains.
    if url.scheme() == "https"
        && (host_matches(host, "messenger.com") && host != "l.messenger.com"
            || facebook_app_host && facebook_app_path)
    {
        Destination::Internal
    } else {
        Destination::External
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversations_and_authentication_stay_in_the_webview() {
        for url in [
            "https://www.messenger.com/",
            "https://www.messenger.com/t/123",
            "https://www.facebook.com/messages/t/123",
            "https://m.facebook.com/login.php?next=https%3A%2F%2Fwww.messenger.com",
            "https://www.facebook.com/login/device-based/regular/login/",
            "https://www.facebook.com/checkpoint/123",
            "https://www.facebook.com/two_step_verification/authentication/",
            "https://www.facebook.com/dialog/oauth?client_id=123",
        ] {
            assert_eq!(
                destination(&Url::parse(url).unwrap()),
                Destination::Internal,
                "{url}"
            );
        }
    }

    #[test]
    fn regular_links_and_lookalike_hosts_open_externally() {
        for url in [
            "https://www.facebook.com/?locale=pl_PL",
            "https://www.facebook.com/some.profile",
            "https://www.facebook.com/watch/",
            "https://www.facebook.com/messages-evil/",
            "https://l.facebook.com/l.php?u=https%3A%2F%2Fexample.com",
            "https://l.messenger.com/l.php?u=https%3A%2F%2Fexample.com",
            "https://www.instagram.com/",
            "https://www.google.com/",
            "https://cdn.fbcdn.net/file.pdf",
            "https://example.com/",
            "https://messenger.com.example.com/",
            "https://evilmessenger.com/",
            "https://facebook.com.example.com/login",
            "http://www.messenger.com/",
            "mailto:person@example.com",
            "tel:+48123456789",
        ] {
            assert_eq!(
                destination(&Url::parse(url).unwrap()),
                Destination::External,
                "{url}"
            );
        }
    }

    #[test]
    fn blobs_and_non_web_schemes_never_reach_the_system_opener() {
        assert_eq!(
            destination(&Url::parse("blob:https://www.messenger.com/123").unwrap()),
            Destination::Blob
        );
        for url in [
            "file:///C:/Windows/test.exe",
            "data:text/html,test",
            "javascript:alert(1)",
            "ms-settings:privacy",
            "about:blank",
        ] {
            assert_eq!(
                destination(&Url::parse(url).unwrap()),
                Destination::Blocked,
                "{url}"
            );
        }
    }
}
