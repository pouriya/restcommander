use rand::{thread_rng, Rng};

const DEFAULT_MAX_LENGTH: usize = 10;

pub struct Captcha {
    max_length: usize,
    captcha_list: Vec<(String, String)>,
}

impl Captcha {
    pub fn new() -> Self {
        Self {
            max_length: DEFAULT_MAX_LENGTH,
            captcha_list: Default::default(),
        }
    }
}

impl Captcha {
    pub fn generate(&mut self, hard: bool) -> (String, String, String) {
        let captcha = captcha::by_name(
            if hard {
                captcha::Difficulty::Medium
            } else {
                captcha::Difficulty::Easy
            },
            match thread_rng().gen_range(0..3) {
                0 => captcha::CaptchaName::Amelia,
                1 => captcha::CaptchaName::Lucy,
                _ => captcha::CaptchaName::Mila,
            },
        );
        let id = uuid::Uuid::new_v4().to_string();
        let characters = captcha.chars().iter().collect::<String>();
        self.captcha_list.push((id.clone(), characters));
        if self.captcha_list.len() > self.max_length {
            self.captcha_list = self.captcha_list.clone().into_iter().skip(1).collect();
        };
        (
            id,
            captcha.chars().iter().collect::<String>(),
            captcha.as_base64().unwrap(),
        )
    }

    pub fn compare_and_update(
        &mut self,
        id: String,
        text: String,
        case_sensitive: bool,
    ) -> bool {
        let text = if case_sensitive {
            text
        } else {
            text.to_lowercase()
        };
        let mut result = false;
        let mut maybe_index = None;
        for (index, (other_id, other_text)) in self.captcha_list.iter().enumerate() {
            if &id == other_id {
                maybe_index = Some(index);
                let other_text = if case_sensitive {
                    other_text.to_owned()
                } else {
                    other_text.to_lowercase()
                };
                if text == other_text {
                    result = true;
                };
                break;
            };
        }
        if let Some(index) = maybe_index {
            self.captcha_list.remove(index);
        };
        result
    }
}
