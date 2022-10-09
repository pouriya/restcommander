use log::error;
use rand::{thread_rng, Rng};
use std::path::PathBuf;
use std::{fs, io};
use thiserror::Error;

const DEFAULT_MAX_LENGTH: usize = 10;

pub struct Captcha {
    filename: PathBuf,
    max_length: usize,
    captcha_list: Vec<(String, String)>,
}

#[derive(Debug, Error)]
pub enum CaptchaError {
    #[error("Could not load captcha from filename {filename:?}")]
    Load {
        filename: PathBuf,
        source: io::Error,
    },
}

impl TryFrom<PathBuf> for Captcha {
    type Error = CaptchaError;

    fn try_from(filename: PathBuf) -> Result<Self, Self::Error> {
        let mut captcha = Self {
            filename,
            max_length: DEFAULT_MAX_LENGTH,
            captcha_list: Default::default(),
        };
        captcha.load()?;
        Ok(captcha)
    }
}

impl Captcha {
    fn load(&mut self) -> Result<(), CaptchaError> {
        if self.filename.is_file() {
            let raw_data =
                fs::read_to_string(self.filename.clone()).map_err(|error| CaptchaError::Load {
                    filename: self.filename.clone(),
                    source: error,
                })?;
            for line in raw_data.lines() {
                if self.captcha_list.len() == self.max_length {
                    break;
                };
                match &line
                    .splitn(2, ' ')
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .clone()[..]
                {
                    [id, text] => {
                        self.captcha_list.push((id.clone(), text.clone()));
                    }
                    _ => {
                        error!("Bad line in CAPTCHA file {:?}: {:?}", self.filename, line)
                    }
                }
            }
            Ok(())
        } else {
            fs::write(self.filename.clone(), "").map_err(|error| CaptchaError::Load {
                filename: self.filename.clone(),
                source: error,
            })
        }
    }

    fn dump(&self) -> Result<(), io::Error> {
        fs::write(
            self.filename.clone(),
            self.captcha_list
                .iter()
                .map(|(id, text)| format!("{} {}", id.to_string(), text.to_string()))
                .collect::<Vec<String>>()
                .join("\n"),
        )
    }
}

impl Captcha {
    pub fn generate(&mut self, hard: bool) -> Result<(String, String, String), io::Error> {
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
        self.dump()?;
        Ok((
            id,
            captcha.chars().iter().collect::<String>(),
            captcha.as_base64().unwrap(),
        ))
    }

    pub fn compare_and_update(&mut self, id: String, text: String, case_sensitive: bool) -> Result<bool, io::Error> {
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
            self.dump()?;
        };
        Ok(result)
    }
}
