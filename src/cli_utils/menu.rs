use crate::cli_utils::CliResult;
use dialoguer::Select;

/// Interactive menu builder
pub struct Menu {
    title: String,
    items: Vec<String>,
}

impl Menu {
    /// Create a new menu with a title
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            items: Vec::new(),
        }
    }

    /// Add an item to the menu
    pub fn item(mut self, label: &str) -> Self {
        self.items.push(label.to_string());
        self
    }

    /// Add multiple items
    pub fn items(mut self, items: Vec<&str>) -> Self {
        self.items.extend(items.iter().map(|s| s.to_string()));
        self
    }

    /// Show the menu and get the selected index
    pub fn interact(&self) -> CliResult<usize> {
        let item_refs: Vec<&str> = self.items.iter().map(|s| s.as_str()).collect();
        Select::new()
            .with_prompt(&self.title)
            .items(&item_refs)
            .default(0)
            .interact()
            .map_err(|e| crate::cli_utils::CliError::IoError(e.to_string()))
    }

    /// Show the menu and get the selected item label
    pub fn interact_label(&self) -> CliResult<String> {
        let idx = self.interact()?;
        Ok(self.items[idx].clone())
    }
}

/// Operation menu (common to all CLIs)
#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
    List,
    View,
    Create,
    Update,
    Delete,
    Cancel,
    Other
}

impl Operation {
    /// Show operation selection menu
    pub fn select() -> CliResult<Self> {
        let menu = Menu::new("Select operation")
            .items(vec!["List", "View", "Create", "Update", "Delete", "Cancel", "Other"]);

        match menu.interact()? {
            0 => Ok(Operation::List),
            1 => Ok(Operation::View),
            2 => Ok(Operation::Create),
            3 => Ok(Operation::Update),
            4 => Ok(Operation::Delete),
            5 => Ok(Operation::Cancel),
            6 => Ok(Operation::Other),
            _ => Ok(Operation::Cancel),
        }
    }

    /// Show operation menu and return selected operation
    pub fn prompt() -> CliResult<Self> {
        Self::select()
    }
}

/// Yes/No confirmation
pub fn confirm_operation(message: &str) -> CliResult<bool> {
    use dialoguer::Confirm;
    Confirm::new()
        .with_prompt(message)
        .interact()
        .map_err(|e| crate::cli_utils::CliError::IoError(e.to_string()))
}
