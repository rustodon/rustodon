use maud::{self, html, Markup, Render};
use rocket::request::Request;
use rocket::response::{self, Responder, Response};
use GIT_REV;

#[derive(Debug, Builder)]
#[builder(setter(into))]
pub struct Page {
    #[builder(default = "None")]
    title: Option<String>,
    content: Markup,
}

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
                title {
                    @if let Some(title) = self.title.as_ref() {
                        "Rustodon | " (title)
                    } @else {
                        "Rustodon"
                    }
                }
            }

            body {
                (self.content)
            }
        }
    }
}
