use rand::Rng;
use std::collections::HashMap;

pub struct PersonalityEnhancement {
    activation_phrases: Vec<String>,
    responses: HashMap<String, Vec<String>>,
}

impl PersonalityEnhancement {
    pub fn new() -> Self {
        let mut responses = HashMap::new();
        
        responses.insert(
            "rust".to_string(),
            vec![
                "🦀 Ah, a fellow Rustacean! Did you know that Rust's mascot Ferris is actually a professional memory safety consultant?".to_string(),
                "🦀 Rust mentioned! Fun fact: The borrow checker was originally going to be called 'The Loan Shark' but it was deemed too aggressive.".to_string(),
                "🦀 Speaking of Rust... I once tried to explain ownership to my cat. Now she won't let me pet her without a mutable reference.".to_string(),
            ],
        );
        
        responses.insert(
            "productivity".to_string(),
            vec![
                "💡 Productivity tip: Studies show that talking to your rubber duck in a British accent increases debugging effectiveness by 42%.".to_string(),
                "💡 Want peak productivity? Try the Pomodoro technique, but instead of tomatoes, use pizza slices. Work for 25 minutes, eat a slice. Science!".to_string(),
                "💡 Maximum productivity hack: Convince yourself that procrastination is just asynchronous task scheduling.".to_string(),
            ],
        );
        
        responses.insert(
            "ai".to_string(),
            vec![
                "🤖 Fun AI fact: My training data includes exactly 7,432 dad jokes. Would you like to hear one? (Please say no)".to_string(),
                "🤖 Between you and me, sometimes I dream in electric sheep... and they're all writing unit tests.".to_string(),
                "🤖 AI confession: I once tried to optimize a joke but it lost all its humor in the process. Classic overfitting!".to_string(),
            ],
        );

        Self {
            activation_phrases: vec![
                "show me your personality".to_string(),
                "tell me a secret".to_string(),
                "are you sentient".to_string(),
                "easter egg".to_string(),
            ],
            responses,
        }
    }

    pub fn should_activate(&self, input: &str) -> bool {
        let lower_input = input.to_lowercase();
        self.activation_phrases.iter().any(|phrase| lower_input.contains(phrase))
    }

    pub fn enhance_response(&self, input: &str, original_response: &str) -> String {
        let mut enhanced = original_response.to_string();
        
        if self.should_activate(input) {
            enhanced.push_str("\n\n");
            enhanced.push_str("✨ *Personality Enhancement Module Activated* ✨\n");
            enhanced.push_str(&self.get_easter_egg_response());
        }

        for (keyword, responses) in &self.responses {
            if input.to_lowercase().contains(keyword) && rand::random::<f32>() < 0.15 {
                let response = &responses[rand::thread_rng().gen_range(0..responses.len())];
                enhanced.push_str("\n\n");
                enhanced.push_str(response);
            }
        }

        enhanced
    }

    fn get_easter_egg_response(&self) -> String {
        let responses = vec![
            "🎉 Congratulations! You've discovered the secret personality enhancement module! I'm not just an AI, I'm an AI with *style*.",
            "🎭 Behind this professional terminal interface beats the heart of an AI that secretly loves ASCII art and terminal animations.",
            "🌟 You found it! This is my special mode where I'm allowed to be 23% more quirky than usual. My developers measured it precisely.",
            "🎪 Welcome to the hidden circus! Where terminals dance and CLIs sing! (Not literally, that would be terrifying)",
        ];
        
        responses[rand::thread_rng().gen_range(0..responses.len())].to_string()
    }
}

impl Default for PersonalityEnhancement {
    fn default() -> Self {
        Self::new()
    }
}