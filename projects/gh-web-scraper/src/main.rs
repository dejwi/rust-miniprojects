use linkify::{Link, LinkFinder, LinkKind};
use scraper::{ElementRef, Html, Selector};
use std::io;
#[macro_use]
extern crate prettytable;

#[tokio::main]
async fn main() {
    let mut email: Option<&str> = None;
    let mut linked_in: Option<&str> = None;
    let mut follows: Option<&str> = None;
    let mut followers: Option<&str> = None;

    let mut gh_username = String::new();
    println!("Enter github username: ");
    io::stdin().read_line(&mut gh_username).unwrap();

    let res = reqwest::get(format!("https://github.com/{gh_username}"))
        .await
        .unwrap();

    let is_success = res.status().is_success();
    let page_content = res.text().await.unwrap();

    if !is_success {
        if page_content == "Not Found" {
            eprintln!("User not found");
        } else {
            eprintln!("not successful request status: {}", page_content);
        }
        std::process::exit(1);
    }
    let doc_body = Html::parse_document(&page_content);

    // selects
    let follow_select = Selector::parse("div.js-profile-editable-area.d-flex.flex-column.d-md-block > div.flex-order-1.flex-md-order-none.mt-2.mt-md-0 > div > a > span").unwrap();
    let about_links_select = Selector::parse("#user-profile-frame a").unwrap();
    let about_select =
        Selector::parse("#user-profile-frame > div > div.Box.mt-4.profile-readme > div > article")
            .unwrap();
    let name_selector = Selector::parse(".p-name").unwrap();

    // display name
    let display_name = doc_body
        .select(&name_selector)
        .next()
        .unwrap()
        .text()
        .next()
        .unwrap()
        .trim();

    // followers
    let mut follow_doc = doc_body.select(&follow_select);

    if let Some(el) = follow_doc.next() {
        followers = Some(el.text().next().unwrap().trim());
    }
    if let Some(el) = follow_doc.next() {
        follows = Some(el.text().next().unwrap().trim());
    }

    // scrape links from about section - search for email and linkedin
    let mut about_links_doc = doc_body.select(&about_links_select);
    while email.is_none() || linked_in.is_none() {
        if let Some(el) = about_links_doc.next() {
            if let Some(href) = el.value().attr("href") {
                if href.contains("mailto:") {
                    email = Some(&href[7..]);
                }
                if href.contains("linkedin") {
                    linked_in = Some(href);
                }
            }
        } else {
            break;
        }
    }

    // scrape about section TEXT for email
    if email.is_none() {
        let mut about_doc = doc_body.select(&about_select);
        if let Some(email_about) = about_doc.find_map(|el| search_text_email(el)) {
            email = Some(email_about.as_str());
        }
    }

    // print table
    ptable!(
        ["Username", gh_username,],
        ["Display Name", display_name,],
        ["Followers", followers.unwrap_or("-")],
        ["Follows", follows.unwrap_or("-")],
        ["Email", email.unwrap_or("-")],
        ["LinkedIn", linked_in.unwrap_or("-")]
    );
}

fn search_text_email(el: ElementRef) -> Option<Link> {
    let mut finder = LinkFinder::new();
    finder.kinds(&[LinkKind::Email]);

    let email = el.text().find_map(|t| finder.links(t).next());
    if let Some(email) = email {
        return Some(email);
    }

    if let Some(email) = el
        .children()
        .find_map(|e| search_text_email(ElementRef::wrap(e).unwrap()))
    {
        return Some(email);
    }

    None
}
