use maud::{self, html, Markup, Render};
use rocket::request::{FlashMessage, Request};
use rocket::response::{self, Responder};
use GIT_REV;

/// Type to store data about a templated page in. Used to insert each page's markup into
/// a base template which sets up stuff like stylesheets and the general html structure.
#[derive(Debug, Default)]
pub struct Page {
    title:   Option<String>,
    content: Option<Markup>,
    flash:   Option<FlashMessage>,
}

impl Page {
    pub fn new() -> Self {
        Self {
            title:   None,
            content: None,
            flash:   None,
        }
    }

    pub fn title<S>(mut self, title: S) -> Self
    where
        S: Into<String>,
    {
        self.title = Some(title.into());
        self
    }

    pub fn content(mut self, content: Markup) -> Self {
        self.content = Some(content);
        self
    }

    pub fn flash(mut self, flash: Option<FlashMessage>) -> Self {
        self.flash = flash;
        self
    }
}

/// Allows returning `Page`s from Rocket routes.
impl<'r> Responder<'r> for Page {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        self.render().respond_to(req)
    }
}

impl Render for Page {
    fn render(&self) -> Markup {
        html! {
            (maud::DOCTYPE)

            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width";

                title {
                    @if let Some(title) = self.title.as_ref() {
                        "Rustodon | " (title)
                    } @else {
                        "Rustodon"
                    }
                }

                link rel="stylesheet" href="https://fonts.googleapis.com/css\
                    ?family=IM+Fell+Great+Primer:400,400i|Nova+Mono";
                link rel="stylesheet" href="/static/style.css";
            }

            body {
                main {
                    @if let Some(flash) = self.flash.as_ref() {
                        div class={"flash " (flash.name())} (flash.msg())
                    }

                    @if let Some(content) = self.content.as_ref() {
                        (content)
                    }
                }

                footer {
                    div {
                        "Running commit "
                        a href=(format!("https://github.com/rustodon/rustodon/commit/{}", GIT_REV))
                            code (GIT_REV)
                        "."
                    }
                }

                script src="/static/js/accessibility.js" {}
            }
        }
    }
}
