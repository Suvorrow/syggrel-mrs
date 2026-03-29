#[derive(Clone, Debug, PartialEq)]
pub enum UiState {
    None,
    DropdownOpen(DropdownType),
    ActiveTool(ActiveToolType),
}

#[derive(Clone, Debug, PartialEq)]
pub enum DropdownType {
    Tools,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ActiveToolType {
    Parser,
}