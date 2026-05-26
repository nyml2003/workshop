use std::fmt::Debug;

pub trait Editor: Debug + Send + Sync {
    fn name(&self) -> &'static str;
    fn launch_cmd(&self) -> &'static str;
}

pub mod editors {
    use super::Editor;

    #[derive(Debug)]
    pub struct Cursor;

    impl Editor for Cursor {
        fn name(&self) -> &'static str {
            "cursor"
        }

        fn launch_cmd(&self) -> &'static str {
            "cursor"
        }
    }

    #[derive(Debug)]
    pub struct VsCode;

    impl Editor for VsCode {
        fn name(&self) -> &'static str {
            "vscode"
        }

        fn launch_cmd(&self) -> &'static str {
            "code"
        }
    }
}

#[derive(Default)]
pub struct EditorRegistry {
    editors: Vec<Box<dyn Editor>>,
}

impl EditorRegistry {
    pub fn new() -> Self {
        Self {
            editors: vec![
                Box::new(editors::Cursor),
                Box::new(editors::VsCode),
            ],
        }
    }

    pub fn find(&self, name: &str) -> Option<&dyn Editor> {
        self.editors.iter().find(|e| e.name() == name).map(|e| e.as_ref())
    }

    pub fn find_or_default(&self, name: &str) -> Option<&dyn Editor> {
        self.editors.len().checked_sub(1).and(self.find(name).or_else(|| self.editors.first().map(|e| e.as_ref())))
    }
}
