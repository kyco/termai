use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Model {
    Gpt4o,
}

impl Model {
    pub fn to_string(&self) -> String {
        match self {
            Model::Gpt4o => "gpt-4o".to_owned(),
        }
    }
}
