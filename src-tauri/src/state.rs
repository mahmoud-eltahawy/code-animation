use std::sync::Mutex;

use itertools::Itertools;
use similar::{utils::diff_lines, Algorithm, ChangeTag};

pub struct AppState {
    pub code: Mutex<Vec<String>>,
    pub markdown: Mutex<Vec<String>>,
}

pub const NEW_LINE: &str = "THENEWLINESYMPOLE";

impl AppState {
    pub fn init() -> Self {
        AppState {
            code: Mutex::new(vec![]),
            markdown: Mutex::new(vec![]),
        }
    }
    #[inline(always)]
    pub fn compare_markdown(&self, markdown: Vec<String>) -> Vec<(String, String)> {
        let new_markdown = self.get_markdown(&markdown);
        self.set_markdown(markdown);
        new_markdown
    }
    #[inline(always)]
    pub fn compare_code(&self, code: Vec<String>) -> Vec<(String, String)> {
        let new_code = self.get_old_code(&code);
        self.set_old_code(code);
        new_code
    }
    fn set_old_code(&self, spans: Vec<String>) {
        *self.code.lock().unwrap() = spans;
    }
    fn set_markdown(&self, spans: Vec<String>) {
        *self.markdown.lock().unwrap() = spans;
    }

    fn get_markdown(&self, markdown: &[String]) -> Vec<(String, String)> {
        let old_markdown = self.markdown.lock().unwrap();
        let lines1 = old_markdown
            .clone()
            .into_iter()
            .intersperse("\n".to_string())
            .collect::<String>();
        let lines2 = markdown
            .iter()
            .intersperse(&"\n".to_string())
            .cloned()
            .collect::<String>();
        let diffs = diff_lines(Algorithm::Myers, &lines1, &lines2)
            .into_iter()
            .flat_map(|(tag, str)| match tag {
                ChangeTag::Delete => {
                    Some(("-1".to_string(), str.replace(NEW_LINE, "\n").to_string()))
                }
                ChangeTag::Insert => {
                    Some(("1".to_string(), str.replace(NEW_LINE, "\n").to_string()))
                }
                ChangeTag::Equal => None,
            })
            .collect::<Vec<_>>();
        diffs
    }

    fn get_old_code(&self, new_code: &[String]) -> Vec<(String, String)> {
        let old_code = self.code.lock().unwrap();
        let lines1 = Itertools::intersperse(old_code.iter(), &"\n".to_string())
            .cloned()
            .collect::<String>();
        let lines2 = Itertools::intersperse(new_code.iter(), &"\n".to_string())
            .cloned()
            .collect::<String>();
        let diffs = diff_lines(Algorithm::Myers, &lines1, &lines2)
            .into_iter()
            .flat_map(|(tag, str)| match tag {
                ChangeTag::Delete => {
                    Some(("-1".to_string(), str.replace(NEW_LINE, "\n").to_string()))
                }
                ChangeTag::Insert => {
                    Some(("1".to_string(), str.replace(NEW_LINE, "\n").to_string()))
                }
                ChangeTag::Equal => None,
            })
            .collect::<Vec<_>>();
        diffs
    }
}
