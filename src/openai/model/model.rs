use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Model {
    Gpt4o,
    Gpt4oMini,
    Gpt4Turbo,
    Gpt4,
    Gpt35Turbo,
    DallE2,
    DallE3,
    Tts1,
    Tts1Hd,
    Whisper1,
    WhisperV2Large,
    TextEmbedding3Large,
    TextEmbedding3Small,
    TextEmbeddingAda002,
    OmniModerationLatest,
    TextModerationLatest,
    TextModerationStable,
    TextModeration007,
    O1Preview,
    O1Mini,
    Gpt4oRealtimePreview,
    Gpt4o20240806,
    Gpt4o20240513,
    Gpt4oMini20240718,
    Gpt4Turbo20240409,
    Gpt4TurboPreview,
    Gpt4Preview,
    Gpt4_0613,
    Gpt4_0314,
    Gpt35Turbo0125,
    Gpt35Turbo1106,
    Gpt35TurboInstruct,
    Babbage002,
    Davinci002,
}

impl Model {
    pub fn to_string(&self) -> String {
        match self {
            Model::Gpt4o => "gpt-4o".to_owned(),
            Model::Gpt4oMini => "gpt-4o-mini".to_owned(),
            Model::Gpt4Turbo => "gpt-4-turbo".to_owned(),
            Model::Gpt4 => "gpt-4".to_owned(),
            Model::Gpt35Turbo => "gpt-3.5-turbo".to_owned(),
            Model::DallE2 => "dall-e-2".to_owned(),
            Model::DallE3 => "dall-e-3".to_owned(),
            Model::Tts1 => "tts-1".to_owned(),
            Model::Tts1Hd => "tts-1-hd".to_owned(),
            Model::Whisper1 => "whisper-1".to_owned(),
            Model::WhisperV2Large => "whisper-v2-large".to_owned(),
            Model::TextEmbedding3Large => "text-embedding-3-large".to_owned(),
            Model::TextEmbedding3Small => "text-embedding-3-small".to_owned(),
            Model::TextEmbeddingAda002 => "text-embedding-ada-002".to_owned(),
            Model::OmniModerationLatest => "omni-moderation-latest".to_owned(),
            Model::TextModerationLatest => "text-moderation-latest".to_owned(),
            Model::TextModerationStable => "text-moderation-stable".to_owned(),
            Model::TextModeration007 => "text-moderation-007".to_owned(),
            Model::O1Preview => "o1-preview".to_owned(),
            Model::O1Mini => "o1-mini".to_owned(),
            Model::Gpt4oRealtimePreview => "gpt-4o-realtime-preview".to_owned(),
            Model::Gpt4o20240806 => "gpt-4o-2024-08-06".to_owned(),
            Model::Gpt4o20240513 => "gpt-4o-2024-05-13".to_owned(),
            Model::Gpt4oMini20240718 => "gpt-4o-mini-2024-07-18".to_owned(),
            Model::Gpt4Turbo20240409 => "gpt-4-turbo-2024-04-09".to_owned(),
            Model::Gpt4TurboPreview => "gpt-4-turbo-preview".to_owned(),
            Model::Gpt4Preview => "gpt-4-0125-preview".to_owned(),
            Model::Gpt4_0613 => "gpt-4-0613".to_owned(),
            Model::Gpt4_0314 => "gpt-4-0314".to_owned(),
            Model::Gpt35Turbo0125 => "gpt-3.5-turbo-0125".to_owned(),
            Model::Gpt35Turbo1106 => "gpt-3.5-turbo-1106".to_owned(),
            Model::Gpt35TurboInstruct => "gpt-3.5-turbo-instruct".to_owned(),
            Model::Babbage002 => "babbage-002".to_owned(),
            Model::Davinci002 => "davinci-002".to_owned(),
        }
    }
}
