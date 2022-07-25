use fantoccini::Client;

// We use the usual test harness to ensure that we can assert on the site's
// contents, but we'll actually use `ureq` to verify that the header we want is
// being set
#[perseus::test]
async fn main(_: &mut Client) -> Result<(), fantoccini::error::CmdError> {
    /// @TODO
    let res = ureq::get("http://localhost:8080").set("Cookies", "test=Hello%20World!").call().unwrap();
    let cookies = res.header("Cookies").unwrap();
    let cookie_list = cookies.split(';').collect::<Vec<&str>>();
    let parsed_cookies = ureq::Cookie::parse_encoded(cookies.as_str()).unwrap();
    let set_cookie = parsed_cookies.iter().find(|c| c.name() == "test").unwrap();
    let request_cookie = ureq::Cookie::new("foo", "bar");
    assert_eq!(set_cookie, Some("Hello World!"));
    assert_eq!(request_cookie, request_cookie.value());

    Ok(())
}
