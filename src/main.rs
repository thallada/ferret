extern crate chan;
extern crate cursive;
extern crate reqwest;
extern crate scraper;
extern crate termion;

use std::process::{Command, Stdio};

use cursive::Cursive;
use cursive::align::HAlign;
use cursive::event::{EventResult, Key};
use cursive::theme::{BaseColor, Color, Theme};
use cursive::traits::*;
use cursive::views::{Dialog, EditView, OnEventView, SelectView};

use reqwest::Client;

use scraper::{Html, Selector};

struct SearchResult {
    title: String,
    domain: String,
    url: String,
    snippet: String,
}

const DDG_HTML_URL: &str = "https://duckduckgo.com";

fn main() {
    let mut siv = Cursive::new();
    siv.add_active_screen();

    let theme = get_dark_theme(&siv);
    siv.set_theme(theme);

    siv.add_layer(
        Dialog::new()
            .title("Ferret")
            // Padding is (left, right, top, bottom)
            .padding((1, 1, 1, 0))
            .content(
                EditView::new()
                    .on_submit(search)
                    .with_id("query")
                    .fixed_width(50),
            )
            .button("Ok", |s| {
                // This will run the given closure, *ONLY* if a view with the
                // correct type and the given ID is found.
                let query = s.call_on_id("query", |view: &mut EditView| {
                    // We can return content from the closure!
                    view.get_content()
                }).unwrap();

                // Run the next step
                search(s, &query);
            }),
    );

    siv.run();
}

// This will make the search and display results in a new popup.
// If the query is empty, we'll show an error message instead.
fn search(s: &mut Cursive, query: &str) {
    if query.is_empty() {
        s.add_layer(Dialog::info("Please enter a search query!"));
    } else {
        let client = Client::new();
        let url = format!("{}/html/?q={}", DDG_HTML_URL, query);
        let mut resp = client.get(&url).send().unwrap();
        let document = Html::parse_document(&resp.text().unwrap());
        let result_selector = Selector::parse(".web-result").unwrap();
        let result_title_selector = Selector::parse(".result__a").unwrap();
        let result_url_selector = Selector::parse(".result__url").unwrap();
        let result_snippet_selector = Selector::parse(".result__snippet").unwrap();

        let mut results: Vec<SearchResult> = Vec::new();
        for result in document.select(&result_selector) {
            let result_title = result.select(&result_title_selector).next().unwrap();
            let result_url = result.select(&result_url_selector).next().unwrap();
            let result_snippet = result.select(&result_snippet_selector).next().unwrap();
            results.push(SearchResult {
                title: result_title.text().collect::<Vec<_>>().join(""),
                domain: result_url.text().collect::<Vec<_>>().join(""),
                url: String::from(result_url.value().attr("href").unwrap()),
                snippet: String::from(result_snippet.text().collect::<Vec<_>>().join("")),
            } );
        }

        s.pop_layer();
        s.add_layer(
            Dialog::new()
                .title(query)
                .button("Exit", |s| s.quit())
                .content(build_list(results))
        )
    }
}

fn build_list(results: Vec<SearchResult>) -> OnEventView<SelectView> {
    let mut result_view = SelectView::new().h_align(HAlign::Left);

    for (i, result) in results.into_iter().enumerate() {
        let url = format!("{}{}", DDG_HTML_URL, result.url);
        result_view = result_view.item(
            format!("{}. {}", i + 1, result.title),
            url.clone()
        )
        .item(
            format!("   {}", result.domain.replace("\n", "").trim()),
            url.clone()
        )
        .item(
            format!("   {}", result.snippet.replace("\n", "").trim()),
            url.clone()
        )
		.item(
			"   ",
            url.clone()
        );
    }

    result_view.set_on_submit(open_url);

    let result_view = OnEventView::new(result_view)
        .on_pre_event_inner(Key::Up, |s| {
            let from_bottom = (s.len() - 1) - s.selected_id().unwrap();
            if from_bottom < 3 {
                s.select_up(3 - from_bottom);
            } else {
                s.select_up(4);
            }
            Some(EventResult::Consumed(None))
        })
        .on_pre_event_inner(Key::Down, |s| {
            if s.selected_id().unwrap() != s.len() - 4 {
                s.select_down(6);
                s.select_up(2);
            }
            Some(EventResult::Consumed(None))
        });

    result_view
}

fn open_url(_: &mut Cursive, url: &str) {
    Command::new("w3m")
        .args(&[url])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .output().unwrap();
}

fn get_dark_theme(siv: &Cursive) -> Theme {
    // We'll return the current theme with a small modification
    let mut theme = siv.current_theme().clone();

    theme.colors.background = Color::TerminalDefault;
    theme.colors.view = Color::Dark(BaseColor::Black);
    theme.colors.shadow = Color::Dark(BaseColor::Black);
    theme.colors.primary = Color::Dark(BaseColor::White);
    theme.colors.tertiary = Color::Dark(BaseColor::Black);

    theme
}
