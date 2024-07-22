use serde::Deserialize;

#[derive(Deserialize, Clone)]
#[allow(dead_code)]
pub struct Har {
    pub log: Log,
}

#[derive(Deserialize, Clone)]
#[allow(dead_code)]
pub struct Log {
    pub entries: Vec<Entry>,
}

#[derive(Deserialize, Clone)]
#[allow(dead_code)]
pub struct Entry {
    pub request: Request,
    pub response: Response,
}

#[derive(Deserialize, Clone)]
#[allow(dead_code)]
pub struct Request {
    pub method: String,
    pub url: String,
    pub headers: Vec<Header>,
    #[serde(rename = "postData")]
    pub post_data: Option<PostData>,
}

#[derive(Deserialize, Clone)]
#[allow(dead_code)]
pub struct Response {
    pub status: u16,
}

#[derive(Deserialize, Clone)]
#[allow(dead_code)]
pub struct Header {
    pub name: String,
    pub value: String,
}

#[derive(Deserialize, Clone)]
#[allow(dead_code)]
pub struct PostData {
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    pub text: String,
}
