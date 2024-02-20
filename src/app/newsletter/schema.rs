use serde::Deserialize;

#[derive(serde::Deserialize)]
pub struct PublishNewsletterRequestBody {
    pub title: String,
    pub content: NewsletterContent,
}

#[derive(Deserialize)]
pub struct NewsletterContent {
    pub html: String,
    pub text: String,
}
