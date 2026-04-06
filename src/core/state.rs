#[derive(Clone, PartialEq, Debug)]
pub enum UiState {
    None,
    DropdownOpen(DropdownType),
    ActiveTool(ActiveToolType),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum DropdownType {
    Tools,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ActiveToolType {
    Parser,
}